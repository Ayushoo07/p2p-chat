// Required libraries and modules from the Rust standard library and libp2p crate.
use std::{error::Error, time::Duration};

use libp2p::{
    // StreamExt provides utilities for working with asynchronous streams.
    futures::StreamExt,
    // Gossipsub is a pub/sub messaging protocol used for decentralized communication.
    gossipsub, 
    // mDNS (Multicast DNS) helps discover peers in the local network.
    mdns, 
    // Noise is a cryptographic protocol for encrypted peer communications.
    noise, 
    // NetworkBehaviour defines the behavior of a node in the network (combining Gossipsub and mDNS).
    swarm::{NetworkBehaviour, SwarmEvent}, 
    // TCP transport protocol, using for peer-to-peer connection.
    tcp, 
    // Yamux is a multiplexing protocol that allows multiple streams over a single connection.
    yamux, 
    // SwarmBuilder is used to create and configure the swarm (the core of peer-to-peer networking).
    SwarmBuilder,
};

// Tokio is an asynchronous runtime that allows the code to run asynchronously.
use tokio::{io, io::AsyncBufReadExt, select};

// Define a custom network behavior by combining Gossipsub and mDNS.
// This macro derives the necessary code to combine the two protocols.
#[derive(NetworkBehaviour)]
pub(crate) struct MyBehaviour {
    // Gossipsub for pub-sub message passing
    gossipsub: gossipsub::Behaviour,
    // mDNS for peer discovery in a local network
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
// The main asynchronous function that starts the P2P node and manages message passing.
async fn main() -> Result<(), Box<dyn Error>> {
    // Create the swarm (P2P node) by building the transport stack and network behaviour.
    let mut swarm = SwarmBuilder::with_new_identity()
        // Use Tokio runtime for asynchronous networking
        .with_tokio()
        // Set up a TCP transport layer with Noise encryption and Yamux multiplexing
        .with_tcp(
            tcp::Config::default(),         // Default TCP transport configuration
            noise::Config::new,             // Secure communication using Noise encryption
            yamux::Config::default,         // Multiplexing using Yamux
        )?
        // Optionally, use QUIC transport (faster, encrypted transport protocol)
        .with_quic()
        // Define the custom behavior (Gossipsub + mDNS) for the P2P node
        .with_behaviour(|key| {
            // Create a default Gossipsub configuration
            let gossipsub_config = gossipsub::Config::default();
            
            // Create a Gossipsub behavior with message signing using the local node's identity key.
            let mut gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()), // Ensure authenticity
                gossipsub_config,                                    // Gossipsub configuration
            )
            .expect("error");

            // Create an mDNS behavior for local peer discovery
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),          // Default mDNS configuration
                key.public().to_peer_id()         // Peer ID is derived from the node's public key
            )?;
            
            // Return the combined behavior for use in the swarm.
            Ok(MyBehaviour { gossipsub, mdns })
        })?
        // Set the swarm configuration with an idle connection timeout of 60 seconds
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        // Build and return the fully configured swarm object
        .build();

    // Create a Gossipsub topic that all peers will subscribe to
    let topic = gossipsub::IdentTopic::new("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

    // Subscribe to the created topic so that this node can receive and publish messages on it
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Create an asynchronous stdin reader to capture user input
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Instruct the swarm to listen for incoming connections on all interfaces (IP4 over QUIC)
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    // Instruct the swarm to listen for incoming connections over TCP as well
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    // Main event loop: Listen for events and handle them (user input and network events).
    loop {
        // Select between different asynchronous tasks: reading stdin or receiving swarm events
        select! {
            // If there's user input (a line of text from stdin)
            Ok(Some(line)) = stdin.next_line() => {
                // Delay for 2 seconds to give peers time to connect before sending the first message
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                // Publish the input line as a Gossipsub message to the subscribed topic
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), line.as_bytes()) {
                    // If an error occurs while publishing the message, print the error.
                    println!("Publish error: {e:?}");
                }
            }
            // Handle events from the swarm (e.g., peer discovery, message receipt)
            event = swarm.select_next_some() => match event {
                // When mDNS discovers a new peer on the local network
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    // For each discovered peer, print the peer ID and add them to Gossipsub explicitly
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        println!("Added explicit peer: {:?}", peer_id);
                    }
                },
                // When a previously discovered peer's mDNS announcement has expired
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    // For each expired peer, remove them from the Gossipsub peer list
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                // When a Gossipsub message is received from a peer
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,   // The peer that sent the message
                    message_id: id,                // Unique ID of the message
                    message,                       // The actual message content (bytes)
                })) => println!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        // Convert the message from bytes to a readable string and print it
                        String::from_utf8_lossy(&message.data),
                    ),
                // When the local node starts listening on a new network address
                SwarmEvent::NewListenAddr { address, .. } => {
                    // Print the address the local node is listening on
                    println!("Local node is listening on {address}");
                }
                // Catch all other events (not handled explicitly)
                _ => {}
            }
        }
    }
}
