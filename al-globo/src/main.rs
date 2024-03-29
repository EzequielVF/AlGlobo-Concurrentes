use std::env::args;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use actix::Actor;
use actix_rt::Arbiter;

use crate::external_entity::ExternalEntity;
use crate::logger::{Log, Logger};
use crate::payment_processor::{PayProcNewPayment, PaymentProcessor, TouristPackage};
use crate::reader::{ParseTouristPackage, Reader};

mod communication;
mod external_entity;
mod logger;
mod payment_processor;
mod reader;

const PORT_AEROLINEA: &str = "3000";
const PORT_BANCO: &str = "3001";
const PORT_HOTEL: &str = "3002";
const IP: &str = "127.0.0.1";

const LOG_FILE: &str = "logs/procesador-pagos";
const FILE: &str = "archivo.csv";

const TIMEOUT: Duration = Duration::from_secs(10);
const REPLICAS: usize = 3;

fn id_to_ctrladdr(id: usize) -> String {
    "127.0.0.1:1234".to_owned() + &*id.to_string()
}

fn id_to_dataaddr(id: usize) -> String {
    "127.0.0.1:1235".to_owned() + &*id.to_string()
}

struct LeaderElection {
    id: usize,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<usize>>, Condvar)>,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
}

impl LeaderElection {
    fn new(id: usize, flag: Arc<Mutex<bool>>) -> LeaderElection {
        let mut ret = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new())),
        };

        let mut clone = ret.clone();
        thread::spawn(move || clone.responder(flag));

        ret.find_new();
        ret
    }

    fn am_i_leader(&self) -> bool {
        self.get_leader_id() == self.id
    }

    fn get_leader_id(&self) -> usize {
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
        println!(
            "ahora soy el liderrr, {:?}",
            *self.leader_id.0.lock().unwrap()
        );
    }

    fn find_new(&mut self) {
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
            self.make_me_leader()
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
    }

    fn responder(&mut self, flag: Arc<Mutex<bool>>) {
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
                    let mut aux = flag.lock().unwrap();
                    *aux = true;
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
        }
    }
}

#[actix_rt::main]
async fn main() {
    // ********** Leader **********
    let argv = args().collect::<Vec<String>>();

    let my_id = argv[1].parse::<usize>().unwrap();
    let flag = Arc::new(Mutex::new(false));
    let mut leader_election: LeaderElection = LeaderElection::new(my_id, flag.clone());
    let socket = UdpSocket::bind(id_to_dataaddr(my_id)).unwrap();
    let mut buf = [0; 4];

    loop {
        {
            *flag.lock().unwrap() = false;
        }
        if leader_election.am_i_leader() {
            println!("Soy tu líder arranco ejecucion");
            //  ********** Run Logger **********
            // Create channel to comunicate with logger
            let (logger_s, logger_rec) = mpsc::channel();

            // Run logger in a new Arbiter
            let logger_arbiter = Arbiter::new();
            let logger_execution = async move {
                let logger_address = Logger::new(LOG_FILE).start();
                logger_s
                    .send(logger_address)
                    .expect("logger send, problema al enviar!");
            };
            logger_arbiter.spawn(logger_execution);
            let logger_address = logger_rec.recv().unwrap();

            // ********** Run Airline **********
            let (airline_s, airline_rec) = mpsc::channel();
            // Run airline in a new Arbiter
            let airline_logger_address = logger_address.clone();
            let airline_arbiter = Arbiter::new();
            let airline_execution = async move {
                let airline_address =
                    ExternalEntity::new("AIRLINE", IP, PORT_AEROLINEA, airline_logger_address)
                        .start();
                airline_s
                    .send(airline_address)
                    .expect("airline send, problema al enviar!");
            };
            airline_arbiter.spawn(airline_execution);

            // ********** Run Bank **********
            let (bank_s, bank_rec) = mpsc::channel();
            let bank_logger_address = logger_address.clone();
            let bank_arbiter = Arbiter::new();
            let bank_execution = async move {
                let bank_address =
                    ExternalEntity::new("BANK", IP, PORT_BANCO, bank_logger_address).start();
                bank_s
                    .send(bank_address)
                    .expect("bank send, problema al enviar!");
            };
            bank_arbiter.spawn(bank_execution);

            // ********** Run Hotel **********
            let (hotel_s, hotel_rec) = mpsc::channel();
            let hotel_logger_address = logger_address.clone();
            let hotel_arbiter = Arbiter::new();
            let hotel_execution = async move {
                let hotel_address =
                    ExternalEntity::new("HOTEL", IP, PORT_HOTEL, hotel_logger_address).start();
                hotel_s
                    .send(hotel_address)
                    .expect("hotel send, problema al enviar!");
            };
            hotel_arbiter.spawn(hotel_execution);

            // ********** Run Payment Processor *************+
            logger_address.do_send(Log("Iniciando Procesador de Pagos".to_string()));

            let (pp_tx, pp_rx) = mpsc::channel();
            let pp_arbiter = Arbiter::new();
            let pp_logger_address = logger_address.clone();
            let pp_execution = async move {
                let airline_address = airline_rec
                    .recv()
                    .expect("Falló recepción dirección Aerolínea");
                let bank_address = bank_rec.recv().expect("Falló recepción dirección Banco");
                let hotel_address = hotel_rec.recv().expect("Falló recepción dirección Hotel");
                let pp_addr = PaymentProcessor::new(
                    bank_address,
                    airline_address,
                    hotel_address,
                    pp_logger_address,
                )
                .start();
                pp_tx.send(pp_addr).expect("pp send, problema al enviar!");
            };
            pp_arbiter.spawn(pp_execution);
            let pp_addr = pp_rx.recv().unwrap();

            // ********** Run Package Reader *************
            logger_address.do_send(Log("Iniciando Package Reader".to_string()));
            let reader_arbiter = Arbiter::new();
            let reader_logger_address = logger_address.clone();
            let reader_execution = async move {
                let reader_addr = Reader::new(FILE, pp_addr, reader_logger_address).start();
                reader_addr.do_send(ParseTouristPackage());
            };
            reader_arbiter.spawn(reader_execution);

            // TODO: ¿llevar a un nuevo thread?
            while !(*flag.lock().unwrap()) {
                socket
                    .set_read_timeout(None)
                    .expect("socket timeout, problema!");
                let (_size, from) = socket.recv_from(&mut buf).unwrap();
                thread::sleep(Duration::from_secs(3));
                socket
                    .send_to("PONG".as_bytes(), from)
                    .expect("socket send, problema al enviar en PONG");
            }

            pp_arbiter.stop();
            reader_arbiter.stop();
            bank_arbiter.stop();
            hotel_arbiter.stop();
            airline_arbiter.stop();

            /*
            logger_arbiter.stop();
            airline_arbiter.stop();
            bank_arbiter.stop();
            hotel_arbiter.stop();
            pp_arbiter.stop();
            reader_arbiter.stop();
            */
        } else {
            let leader_id = leader_election.get_leader_id();

            socket
                .send_to("PING".as_bytes(), id_to_dataaddr(leader_id))
                .unwrap();
            socket.set_read_timeout(Some(TIMEOUT)).unwrap();

            if let Ok((_size, _from)) = socket.recv_from(&mut buf) {
                // todo
                println!("Me contestó el lider");
            } else {
                println!("NO Me contestó el lider");
                leader_election.find_new()
            }
        }
    }
}
