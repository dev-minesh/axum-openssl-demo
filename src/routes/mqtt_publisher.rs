use paho_mqtt as mqtt;
use serde_json::{json, Value};
use std::{
    env,
    process,
    thread,
    time::{Duration, SystemTime}
};
use futures::executor::block_on;

// extern crate paho_mqtt as mqtt;

const DFLT_BROKER:&str = "mqtt://168.62.42.83:1883";
const DFLT_CLIENT:&str = "mecha-pub";
const DFLT_TOPICS:&[&str] = &["mecha-101", "rust/test"];
// Define the qos.
const QOS:i32 = 1;

fn time_now_hundredths() -> u64 {
    (SystemTime::now()
     .duration_since(SystemTime::UNIX_EPOCH)
     .unwrap()
     .as_millis()
     / 10) as u64
}

pub fn mqtt_pub(payload:Value) {
    // Initialize the logger from the environment
    env_logger::init();

    // Command-line option(s)
    let host = env::args()
        .nth(1)
        .unwrap_or_else(|| "mqtt://168.62.42.83:1883".to_string());

    // Create the client
    let cli = mqtt::AsyncClient::new(host)
    .unwrap_or_else(|err| {
        println!("Error creating the client: {}", err);
        process::exit(1);
    });
    
    if let Err(err) = block_on(async {
        // Connect with default options and wait for it to complete or fail
        // The default is an MQTT v3.x connection.
        println!("Connecting to the MQTT server");
        cli.connect(None).await?;

        // Create a message and publish it
        println!("Publishing a message on the topic 'test'");
        let msg = mqtt::Message::new("mecha-001", payload.to_string(), mqtt::QOS_1);
        cli.publish(msg).await?;

        // Disconnect from the broker
        println!("Disconnecting");
        cli.disconnect(None).await?;

        Ok::<(), mqtt::Error>(())
    }) {
        eprintln!("{}", err);
    }
}

pub fn init_paho_mqtt_1() -> () {
    env_logger::init();
    const TRUST_STORE: &str = "certs/server-ca.crt";
    println!("TRUST STORE :: {}",TRUST_STORE);
    let mut trust_store = env::current_dir().expect("can't access current dir");
    trust_store.push(TRUST_STORE);

    if !trust_store.exists() {
        println!("The trust store file does not exist: {:?}", trust_store);
        process::exit(1);
    }

    let host = env::args().nth(1).unwrap_or_else(||
        DFLT_BROKER.to_string()
    );

    // Define the set of options for the create.
    // Use an ID for a persistent session.
    let create_opts = mqtt::CreateOptionsBuilder::new()
        .server_uri(host)
        .client_id(DFLT_CLIENT.to_string())
        .persistence("persist")
        .finalize();

    // Create a client.
    let cli = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
        println!("Error creating the client: {:?}", err);
        process::exit(1);
    });

    let ssl_opts = mqtt::SslOptionsBuilder::new()
        .trust_store(trust_store).expect("can't access current dir")
        // .key_store(key_store).expect("can't access current dir")
        .finalize();

    // Define the set of options for the connection.
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        //.ssl_options(ssl_opts)
        .user_name("admin")
        .password("Mecha101")
        .keep_alive_interval(Duration::from_secs(20))
        .clean_session(true)
        .finalize();

    // Connect and wait for it to complete or fail.
    if let Err(e) = cli.connect(conn_opts).wait() {
        println!("Unable to connect:\n\t{:?}", e);
        process::exit(1);
    }

    loop {
        let t0 = time_now_hundredths();
        let mut t = t0;

        // Wait until the time reading changes
        while t == t0 {
            thread::sleep(Duration::from_millis(1000));
            t = time_now_hundredths();
        }

        let msg = mqtt::Message::new(DFLT_TOPICS[0], format!("Hello jack"), QOS);
        // We don't need to use `try_publish()` here since we just wait on
        // the token, but this shows how we could use it.
       println!("Message publishing");
        match cli.try_publish(msg) {
            Err(err) => eprintln!("Error creating/queuing the message: {}", err),
            Ok(tok) => {
                println!("Message published");
                if let Err(err) = tok.wait() {
                    eprintln!("Error sending message: {}", err);
                }
            }
        }
        println!("Disconnecting...........");
         // Disconnect from the broker.
        //cli.disconnect(None);
        println!("Disconnect from the broker");
    }
}
