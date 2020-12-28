use std::sync::mpsc::{self, sync_channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use futures::channel::mpsc::UnboundedSender;
use futures_timer::Delay;
use log::*;
use prost_derive::Message;
use rand::Rng;

#[cfg(test)]
pub mod config;
pub mod errors;
pub mod persister;
#[cfg(test)]
mod tests;

use self::errors::*;
use self::persister::*;
use crate::proto::raftpb::*;

/// Generate a random number between 150 and 300
///
/// It is used in hearbeat and election timeout
fn random_timeout() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(150, 301)
}

#[derive(Default)]
pub struct ApplyMsg {
    pub command_valid: bool,
    pub command: Vec<u8>,
    pub command_index: u64,
}

/// State of a raft peer.
#[derive(Clone, Message)]
pub struct State {
    #[prost(uint64)]
    pub term: u64,

    #[prost(uint64, optional)]
    pub voted_for: Option<u64>,

    #[prost(bool)]
    pub is_leader: bool,
}

impl State {
    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.term
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.is_leader
    }

    /// Id of the peer it voted.
    pub fn voted(&self) -> Option<u64> {
        self.voted_for
    }
}

// A single Raft peer.
pub struct Raft {
    // RPC end points of all peers
    peers: Vec<RaftClient>,
    // Object to hold this peer's persisted state
    persister: Box<dyn Persister>,
    // this peer's index into peers[]
    me: usize,
    state: Arc<State>,
    // Your data here (2A, 2B, 2C).
    // Look at the paper's Figure 2 for a description of what
    // state a Raft server must maintain.
    leader_id: u64,

    // Volatile state on all servers.
    commit_index: u64,
    last_applied: u64,

    // Volatile state on leaders.
    next_index: Option<Vec<u64>>,
    match_index: Option<Vec<u64>>,
}

impl Raft {
    /// the service or tester wants to create a Raft server. the ports
    /// of all the Raft servers (including this one) are in peers. this
    /// server's port is peers[me]. all the servers' peers arrays
    /// have the same order. persister is a place for this server to
    /// save its persistent state, and also initially holds the most
    /// recent saved state, if any. apply_ch is a channel on which the
    /// tester or service expects Raft to send ApplyMsg messages.
    /// This method must return quickly.
    pub fn new(
        peers: Vec<RaftClient>,
        me: usize,
        persister: Box<dyn Persister>,
        mut apply_ch: UnboundedSender<ApplyMsg>,
    ) -> Raft {
        let raft_state = persister.raft_state();

        // Your initialization code here (2A, 2B, 2C).
        let mut rf = Raft {
            peers,
            persister,
            me,
            state: Arc::default(),
            leader_id: 0,
            commit_index: 0,
            last_applied: 0,
            next_index: None,
            match_index: None,
        };

        // initialize from state persisted before a crash
        rf.restore(&raft_state);

        apply_ch.start_send(ApplyMsg::default()).unwrap();

        rf
    }

    /// save Raft's persistent state to stable storage,
    /// where it can later be retrieved after a crash and restart.
    /// see paper's Figure 2 for a description of what should be persistent.
    fn persist(&mut self) {
        let mut raft_state: Vec<u8> = vec![];
        labcodec::encode(Arc::get_mut(&mut self.state).unwrap(), &mut raft_state).unwrap();
        self.persister.save_raft_state(raft_state);
    }

    /// restore previously persisted state.
    fn restore(&mut self, data: &[u8]) {
        if data.is_empty() {
            // bootstrap without any state?
            return;
        }
        match labcodec::decode::<State>(data) {
            Ok(o) => {
                self.state = Arc::new(o);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }

    /// Send a RequestVote RPC to a server.
    /// server is the index of the target server in peers.
    /// expects RPC arguments in args.
    ///
    /// The labrpc package simulates a lossy network, in which servers
    /// may be unreachable, and in which requests and replies may be lost.
    /// This method sends a request and waits for a reply. If a reply arrives
    /// within a timeout interval, This method returns Ok(_); otherwise
    /// this method returns Err(_). Thus this method may not return for a while.
    /// An Err(_) return can be caused by a dead server, a live server that
    /// can't be reached, a lost request, or a lost reply.
    ///
    /// This method is guaranteed to return (perhaps after a delay) *except* if
    /// the handler function on the server side does not return.  Thus there
    /// is no need to implement your own timeouts around this method.
    ///
    /// look at the comments in ../labrpc/src/lib.rs for more details.
    fn send_request_vote(
        &self,
        server: usize,
        args: RequestVoteArgs,
    ) -> Receiver<Result<RequestVoteReply>> {
        let peer = &self.peers[server];
        let peer_clone = peer.clone();
        let (tx, rx) = sync_channel::<Result<RequestVoteReply>>(1);
        peer.spawn(async move {
            let res = peer_clone.request_vote(&args).await.map_err(Error::Rpc);
            tx.send(res).unwrap();
        });
        rx
    }

    fn start<M>(&self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        let index = 0;
        let term = 0;
        let is_leader = true;
        let mut buf = vec![];
        labcodec::encode(command, &mut buf).map_err(Error::Encode)?;
        // Your code here (2B).

        if is_leader {
            Ok((index, term))
        } else {
            Err(Error::NotLeader)
        }
    }
}

impl Raft {
    /// Only for suppressing deadcode warnings.
    #[doc(hidden)]
    pub fn __suppress_deadcode(&mut self) {
        let _ = self.start(&0);
        let _ = self.send_request_vote(0, Default::default());
        self.persist();
        let _ = &self.state;
        let _ = &self.me;
        let _ = &self.persister;
        let _ = &self.peers;
    }
}

enum TimeoutMsg {
    Heartbeat,
    Election,
}

// Choose concurrency paradigm.
//
// You can either drive the raft state machine by the rpc framework,
//
// ```rust
// struct Node { raft: Arc<Mutex<Raft>> }
// ```
//
// or spawn a new thread runs the raft state machine and communicate via
// a channel.
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    raft: Arc<Mutex<Raft>>,
    tx: Arc<Mutex<mpsc::Sender<TimeoutMsg>>>,
}

impl Node {
    /// Create a new raft service.
    pub fn new(raft: Raft) -> Self {
        let raft = Arc::new(Mutex::new(raft));
        let raft1 = raft.clone();
        let (tx, rx) = mpsc::channel::<TimeoutMsg>();

        // spawn a background thred that handles timeouts: Election and Heartbeat(only leader)
        thread::spawn(move || loop {
            let raft = &mut raft1.lock().unwrap();

            // Election & Heartbeat timeout
            futures::executor::block_on(async {
                let _election_delay = Delay::new(Duration::from_millis(random_timeout())).await;
                let _hearbeat_delay = Delay::new(Duration::from_millis(200)).await;
                match rx.try_recv() {
                    Ok(msg) => match msg {
                        TimeoutMsg::Election => {}
                        TimeoutMsg::Heartbeat => {}
                    },
                    Err(e) => match e {
                        mpsc::TryRecvError::Disconnected => panic!(e),
                        mpsc::TryRecvError::Empty => {
                            let mut vote_count = 0;
                            let n = raft.peers.len();
                            let state = Arc::get_mut(&mut raft.state).unwrap();
                            // Did not receive a RequestVote even after timeouts.
                            // Become candidate, send out request_vote to other peers.
                            let mut rs = state.clone();
                            for i in 0..n {
                                rs.term += 1;
                                let args = RequestVoteArgs {
                                    term: rs.term,
                                    candidate_id: raft.me as u64,
                                    last_log_term: 0,
                                    last_log_index: 0,
                                };
                                if i != raft.me {
                                    match raft.send_request_vote(i, args).recv() {
                                        Ok(reply) => match reply {
                                            Ok(reply) => {
                                                // Term > CurrentTerm seen, update to the latest
                                                // term.
                                                //
                                                // if reply.term > state.current_term {
                                                //     state.current_term = reply.term;
                                                // }
                                                vote_count +=
                                                    if reply.vote_granted { 1 } else { 0 };
                                            }
                                            Err(e) => error!("{:#?}", e),
                                        },
                                        Err(_) => {}
                                    }
                                }
                            }
                            rs.is_leader = vote_count > (n / 2 + n % 2);
                        }
                    },
                }
            });
        });
        Self {
            raft,
            tx: Arc::new(Mutex::new(tx)),
        }
    }

    /// the service using Raft (e.g. a k/v server) wants to start
    /// agreement on the next command to be appended to Raft's log. if this
    /// server isn't the leader, returns [`Error::NotLeader`]. otherwise start
    /// the agreement and return immediately. there is no guarantee that this
    /// command will ever be committed to the Raft log, since the leader
    /// may fail or lose an election. even if the Raft instance has been killed,
    /// this function should return gracefully.
    ///
    /// the first value of the tuple is the index that the command will appear
    /// at if it's ever committed. the second is the current term.
    ///
    /// This method must return without blocking on the raft.
    pub fn start<M>(&self, command: &M) -> Result<(u64, u64)>
    where
        M: labcodec::Message,
    {
        self.raft.lock().unwrap().start(command)
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.raft.lock().unwrap().state.term
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        let raft = self.raft.lock().unwrap();
        raft.leader_id == raft.me as u64
    }

    /// The current state of this peer.
    pub fn get_state(&self) -> State {
        State {
            term: self.term(),
            is_leader: self.is_leader(),
            voted_for: None,
        }
    }

    /// the tester calls kill() when a Raft instance won't be
    /// needed again. you are not required to do anything in
    /// kill(), but it might be convenient to (for example)
    /// turn off debug output from this instance.
    /// In Raft paper, a server crash is a PHYSICAL crash,
    /// A.K.A all resources are reset. But we are simulating
    /// a VIRTUAL crash in tester, so take care of background
    /// threads you generated with this Raft Node.
    pub fn kill(&self) {
        // Your code here, if desired.
    }
}

#[async_trait::async_trait]
impl RaftService for Node {
    /// RequestVote RPC handler.
    async fn request_vote(&self, _args: RequestVoteArgs) -> labrpc::Result<RequestVoteReply> {
        // TODO RequestVote received, process
        self.tx
            .lock()
            .unwrap()
            .send(TimeoutMsg::Election)
            .map_err(|_| labrpc::Error::Stopped)?;
        Ok(RequestVoteReply {
            term: 0,
            vote_granted: false,
        })
    }

    /// AppendEntries RPC handler.
    async fn append_entries(&self, _args: AppendEntriesArgs) -> labrpc::Result<AppendEntriesReply> {
        // TODO Heartbeat received, process
        Ok(AppendEntriesReply {
            success: false,
            term: 0,
        })
    }
}
