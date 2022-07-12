use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use actix::Addr;

use crate::{Reader};
use crate::reader::{StartReading, StopReading};

const REPLICAS: usize = 3;
pub const TIMEOUT: Duration = Duration::from_secs(10);


fn id_to_ctrladdr(id: usize) -> String {
    "127.0.0.1:1234".to_owned() + &*id.to_string()
}

pub fn id_to_dataaddr(id: usize) -> String {
    "127.0.0.1:1235".to_owned() + &*id.to_string()
}

pub struct LeaderElection {
    id: usize,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<usize>>, Condvar)>,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
    reader_addr: Addr<Reader>,
}

impl LeaderElection {
    pub(crate) fn new(id: usize, reader_addr: Addr<Reader>) -> LeaderElection {
        let mut ret = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new())),
            reader_addr,
        };

        let mut clone = ret.clone();
        thread::spawn(move || clone.responder());

        ret.find_new();
        ret
    }

    pub(crate) fn am_i_leader(&self) -> bool {
        let am_i = self.get_leader_id() == self.id;
        println!("¿Soy el lider? {}", am_i);

        am_i
    }

    pub(crate) fn get_leader_id(&self) -> usize {
        self.leader_id
            .1
            .wait_while(self.leader_id.0.lock().unwrap(), |leader_id| {
                leader_id.is_none()
            })
            .unwrap()
            .unwrap()
    }

    fn id_to_msg(&self, header: u8) -> Vec<u8> {
        let mut msg = vec![header];
        msg.extend_from_slice(&self.id.to_le_bytes());
        msg
    }

    fn send_election(&self) {
        // P envía el mensaje ELECTION a todos los procesos que tengan número mayor
        let msg = self.id_to_msg(b'E');
        for replica_id in (self.id + 1)..REPLICAS {
            self.socket
                .send_to(&msg, id_to_ctrladdr(replica_id))
                .unwrap();
        }
    }

    fn make_me_leader(&self) {
        // El nuevo coordinador se anuncia con un mensaje COORDINATOR
        println!("[{}] me anuncio como lider", self.id);
        let msg = self.id_to_msg(b'C');
        for peer_id in 0..REPLICAS {
            if peer_id != self.id {
                self.socket.send_to(&msg, id_to_ctrladdr(peer_id)).unwrap();
            }
        }
        *self.leader_id.0.lock().unwrap() = Some(self.id);

        println!("ahora soy el liderrr, {:?}", *self.leader_id.0.lock().unwrap());
        self.reader_addr.do_send(StartReading());
    }

    pub(crate) fn find_new(&mut self) {
        println!("Entro a find new");
        if *self.stop.0.lock().unwrap() {
            return;
        }
        if self.leader_id.0.lock().unwrap().is_none() {
            // ya esta buscando lider
            return;
        }
        println!("[{}] buscando lider", self.id);
        *self.got_ok.0.lock().unwrap() = false;
        *self.leader_id.0.lock().unwrap() = None;

        // P envía el mensaje ELECTION a todos los procesos que tengan número mayor
        self.send_election();

        // 2. Si nadie responde, P gana la elección y es el nuevo coordinador
        let got_ok =
            self.got_ok
                .1
                .wait_timeout_while(self.got_ok.0.lock().unwrap(), TIMEOUT, |got_it| !*got_it);
        println!("llegue hasta aca");
        if !*got_ok.unwrap().0 {
            println!("Me falta entrar acá");
            self.make_me_leader();
            println!("sali de make me leader");
        } else {
            // 3. Si contesta algún proceso con número mayor, éste continúa con el proceso y P finaliza
            let _lock = self
                .leader_id
                .1
                .wait_while(self.leader_id.0.lock().unwrap(), |leader_id| {
                    leader_id.is_none()
                })
                .expect("condvar, problema con el wait-while!");
        }
        println!("termino find new, fin de funcion");
    }

    fn responder(&mut self) {
        while !*self.stop.0.lock().unwrap() {
            let mut buf = [0; size_of::<usize>() + 1];
            let (_size, _from) = self.socket.recv_from(&mut buf).unwrap();
            let id_from = usize::from_le_bytes(buf[1..].try_into().unwrap());
            if *self.stop.0.lock().unwrap() {
                break;
            }
            match &buf[0] {
                b'O' => {
                    println!("[{}] recibí OK de {}", self.id, id_from);
                    *self.got_ok.0.lock().unwrap() = true;
                    self.got_ok.1.notify_all();
                }
                b'E' => {
                    println!("[{}] recibí Election de {}", self.id, id_from);
                    if id_from < self.id {
                        self.socket
                            .send_to(&self.id_to_msg(b'O'), id_to_ctrladdr(id_from))
                            .unwrap();
                        let mut me = self.clone();
                        thread::spawn(move || me.find_new());
                    }
                }
                b'C' => {
                    println!("[{}] recibí nuevo coordinador {}", self.id, id_from);
                    /*let mut aux = flag.lock().unwrap();
                    *aux = true;*/

                    self.reader_addr.do_send(StopReading());

                    *self.leader_id.0.lock().unwrap() = Some(id_from);
                    self.leader_id.1.notify_all();
                }
                _ => {
                    println!("[{}] ??? {}", self.id, id_from);
                }
            }
        }
        *self.stop.0.lock().unwrap() = false;
        self.stop.1.notify_all();
    }

    #[allow(dead_code)]
    fn stop(&mut self) {
        *self.stop.0.lock().unwrap() = true;
        let _lock = self
            .stop
            .1
            .wait_while(self.stop.0.lock().unwrap(), |should_stop| *should_stop)
            .expect("condvar, problema con el wait-while!");
    }

    fn clone(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_id: self.leader_id.clone(),
            got_ok: self.got_ok.clone(),
            stop: self.stop.clone(),
            reader_addr: self.reader_addr.clone(),
        }
    }
}
