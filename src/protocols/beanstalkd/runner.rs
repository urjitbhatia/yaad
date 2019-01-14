use protocols;
use settings;

pub fn run(conf: settings::Settings) {
    info!("Running tcp listener");

    match conf.addr {
        Some(addr) => {
            let beanstalkd_srv = protocols::beanstalkd::Beanstalkd {};
            match beanstalkd_srv.listen_and_serve(addr) {
                Ok(_) => info!("beanstalkd done ok"),
                Err(e) => warn!("beanstalkd error: {}", e),
            }
        }
        None => info!("beanstalkd runner error: socket addr not specified"),
    }
}
