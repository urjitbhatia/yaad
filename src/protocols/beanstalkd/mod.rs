pub use self::runner::run;

use self::codec::Lines;
use super::super::yaad::hub;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

mod codec;
mod runner;

struct Beanstalkd {}

impl Beanstalkd {
    /// Use yaad to run a beanstalkd emulation
    pub fn listen_and_serve(self, addr: SocketAddr) -> io::Result<()> {
        info!("Staring beanstalkd protocol at {:?}", addr);

        // listen to given address for new client connections
        let socket = TcpListener::bind(&addr)?;

        // Create a yaad hub instance
        let h = Arc::new(Mutex::new(hub::Hub::new(10_000)));

        let done = socket
            .incoming()
            .map_err(|e| error!("failed to accept new connection: {:?}", e))
            .for_each(move |listener| {
                info!("Connected to a client");
                handle_client(listener, Arc::clone(&h));
                Ok(())
            });

        tokio::run(done);
        Ok(())
    }
}

fn handle_client(tcp_stream: TcpStream, _hub: Arc<Mutex<hub::Hub>>) {
    // Wrap the socket with the `Lines` codec
    let lines = Lines::new(tcp_stream);

    // We use the `into_future` combinator to extract the first
    // item from the lines stream. `into_future` takes a `Stream`
    // and converts it to a future of `(first, rest)` where `rest`
    // is the original stream instance.
    let connection = lines
        .into_future()
        // `into_future` doesn't have the right error type, so map
        // the error to make it work.
        .map_err(|(e, _)| {
            error!("Error handling connection: {:?}", e);
            e
        })
        // Process the first received line as the command.
        .and_then(|(command, _lines)| {
            info!("handling data");
            let cmd = match command {
                Some(cmd) => cmd,
                None => {
                    // The remote client closed the connection without
                    // sending any data.
                    info!("Found no valid command");
                    return future::Either::A(future::ok(()));
                }
            };
            info!("Running command: `{:?}`", cmd);

            future::Either::B(future::ok(()))
        })
        // Task futures have an error of type `()`, this ensures we handle the
        // error. We do this by printing the error to STDOUT.
        .map_err(|e| {
            println!("connection error = {:?}", e);
        });

    tokio::spawn(connection);
}
