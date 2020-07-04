use anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{self, Deserializer};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

pub type Result<T> = anyhow::Result<T>;

// This constant is used for invoking log compaction
const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are persisted to disk in log files. Log files are named after
/// monotonically increasing generation numbers with a `log` extension name.
/// A `BTreeMap` in memory stores the keys and the value locations for fast query.
///
/// ```rust
/// # use yakv::{KvStore, Result};
/// # fn try_main() -> Result<()> {
/// use std::env::current_dir;
/// let mut store = KvStore::open(current_dir()?)?;
/// store.set("key".to_owned(), "value".to_owned())?;
/// let val = store.get("key".to_owne())?;
/// assert_eq!(val, Some("value".to_owned()));
/// # Ok(())
/// # }
/// ```
pub struct KvStore {
    path: PathBuf,
    current_id: u64,
    writer: BufWriterWithPos<File>,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    index: BTreeMap<String, CommandPos>,
    stale_data: u64,
}

impl KvStore {
    /// Opens a KvStore with the given path.
    pub fn open<T: Into<PathBuf>>(path: T) -> Result<Self> {
        // try to load all log files in the given path
        // if it failed then create a log file with an id suffix-ed to the file
        // e.g. key-1.log, key-2.log, key-3.log, etc
        // after loading all the logs, build the index in-memory
        let path = path.into();
        fs::create_dir_all(&path)?;

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();
        let mut stale_data = 0;

        let ids = sorted_ids(&path)?;
        // println!("IDS: {:?}", ids);
        for &id in &ids {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, id))?)?;
            stale_data += load_log(id, &mut reader, &mut index)?;
            readers.insert(id, reader);
        }

        let current_id = ids.last().unwrap_or(&0) + 1;
        let writer = create_log_file(current_id, &path, &mut readers)?;

        Ok(KvStore {
            path,
            current_id,
            writer,
            readers,
            index,
            stale_data,
        })
    }

    /// Sets the value of s string key to a string.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;

        if let Command::Set { key, .. } = cmd {
            if let Some(old_cmd) = self.index.insert(
                key,
                CommandPos::from((self.current_id, pos..self.writer.pos)),
            ) {
                self.stale_data += old_cmd.len;
            }
        }

        // Handle log compaction
        if self.stale_data > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    /// Gets the string value for a given key.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // println!("{:?}", self.index);
        if let Some(cmd_pos) = self.index.get(&key) {
            let reader = self
                .readers
                .get_mut(&cmd_pos.id)
                .expect("Cannot find reader");

            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);
            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                return Ok(Some(value));
            } else {
                return Err(anyhow::Error::msg("Unexpected command"));
            }
        }
        Ok(None)
    }

    /// Removes the given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        // check if key exist in index and delete if from the log file
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key.to_owned());
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            let old_cmd = self.index.remove(&key).expect("Key not found");
            self.stale_data += old_cmd.len;
            Ok(())
        } else {
            Err(anyhow::Error::msg("Key not found"))
        }
    }

    fn compact(&mut self) -> Result<()> {
        // increment id by 1
        // this will be used by compaction writer
        let compaction_id = self.current_id + 1;
        self.current_id += 2;
        self.writer = create_log_file(self.current_id, &self.path, &mut self.readers)?;
        let mut compaction_writer = create_log_file(compaction_id, &self.path, &mut self.readers)?;

        let mut new_pos = 0;
        for cmd_pos in &mut self.index.values_mut() {
            let cmd_reader = self.readers.get_mut(&cmd_pos.id).expect("reader not found");
            if cmd_reader.pos != cmd_pos.pos {
                cmd_reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut cmd_reader = cmd_reader.take(cmd_pos.len);
            let len = io::copy(&mut cmd_reader, &mut compaction_writer)?;
            *cmd_pos = CommandPos::from((compaction_id, new_pos..new_pos + len));
            new_pos += len;
        }
        compaction_writer.flush()?;

        let stale_ids: Vec<_> = self
            .readers
            .keys()
            .filter(|id| id < &&compaction_id)
            .cloned()
            .collect();

        for stale_id in stale_ids {
            self.readers.remove(&stale_id);
            fs::remove_file(log_path(&self.path, stale_id))?;
        }
        self.stale_data = 0;

        Ok(())
    }
}

fn log_path<T: AsRef<Path>>(path: T, id: u64) -> PathBuf {
    path.as_ref().join(format!("{}.log", id))
}

fn create_log_file(
    id: u64,
    path: &Path,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>> {
    let path = log_path(&path, id);
    let writer = BufWriterWithPos::new(OpenOptions::new().create(true).append(true).open(&path)?)?;
    readers.insert(id, BufReaderWithPos::new(File::open(&path)?)?);
    Ok(writer)
}

// load a log and build index
fn load_log(
    id: u64,
    reader: &mut BufReaderWithPos<File>,
    index: &mut BTreeMap<String, CommandPos>,
) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut stale_data = 0;
    // println!("ID: {}", id);
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.insert(key, CommandPos::from((id, pos..new_pos))) {
                    stale_data += old_cmd.len;
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    stale_data += old_cmd.len;
                }

                stale_data += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(stale_data)
}

// get all ids from the log files in a given path
//
// Returns sorted id numbers
fn sorted_ids(path: &Path) -> Result<Vec<u64>> {
    let mut ids: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|dir_entry| -> Result<_> { Ok(dir_entry?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .filter_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    ids.sort();
    Ok(ids)
}

pub struct BufReaderWithPos<T: Read + Seek> {
    reader: BufReader<T>,
    pos: u64,
}

impl<T: Read + Seek> BufReaderWithPos<T> {
    fn new(mut file: T) -> Result<Self> {
        let pos = file.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(file),
            pos,
        })
    }
}

impl<T: Read + Seek> Read for BufReaderWithPos<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<T: Read + Seek> Seek for BufReaderWithPos<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<T: Write + Seek> {
    writer: BufWriter<T>,
    pos: u64,
}

impl<T: Write + Seek> BufWriterWithPos<T> {
    fn new(mut file: T) -> Result<Self> {
        let pos = file.seek(SeekFrom::Current(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(file),
            pos,
        })
    }
}

impl<T: Write + Seek> Write for BufWriterWithPos<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<T: Write + Seek> Seek for BufWriterWithPos<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

/// Represent KV store commands
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Self {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Self {
        Command::Remove { key }
    }
}

/// Position for Command in log file
///
/// Stores log file id, offset, and length
#[derive(Debug)]
struct CommandPos {
    id: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((id, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            id,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}
