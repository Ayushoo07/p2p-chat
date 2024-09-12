# Libp2p Gossipsub & mDNS Example

This project demonstrates a basic peer-to-peer (P2P) application using [libp2p](https://libp2p.io/) in Rust, combining two key components:
- **Gossipsub** for decentralized message passing (pub-sub messaging).
- **mDNS** for local peer discovery without needing manual peer configuration.

The application allows nodes to communicate with each other asynchronously, passing messages over a Gossipsub topic, while automatically discovering peers on the same local network using mDNS.

## Features

- **Gossipsub (Pub-Sub Messaging)**: Enables peers to subscribe to topics and exchange messages over those topics.
- **mDNS (Multicast DNS)**: Automatically discovers peers on the same local network and adds them to the Gossipsub network for communication.
- **Secure Communication**: All communications between peers are encrypted using the [Noise protocol](https://noiseprotocol.org/).
- **Multiplexing**: Multiple streams of data are handled over the same connection using Yamux.

## Requirements

- **Rust**: Make sure Rust is installed on your system. You can install Rust [here](https://www.rust-lang.org/tools/install).
- **Tokio**: The asynchronous runtime used in this project. It's included in the `Cargo.toml` dependencies.

## Installation

1. Clone the repository:

    ```bash
    git clone https://github.com/yourusername/libp2p-chat-example.git
    cd libp2p-chat-example
    ```

2. Install the required dependencies via `cargo`:

    ```bash
    cargo build
    ```

## How It Works

### Overview

- The program listens on both TCP and QUIC for peer-to-peer connections.
- It uses **Gossipsub** to subscribe to a topic and send/receive messages over that topic.
- **mDNS** is used to automatically find other peers on the same local network and add them to the network topology.
- Messages are sent by typing them into the terminal, and received messages are printed to the console.

### Key Components

1. **Gossipsub**: 
    - Allows pub-sub messaging where peers subscribe to a common topic and exchange messages.
    - Each message is signed by the peer sending it to ensure authenticity.

2. **mDNS**:
    - Discovers peers on the local network and automatically adds them to the peer list for Gossipsub.

3. **Swarm**:
    - The swarm represents the node itself and handles the network events, including peer discovery, message passing, and connection management.

## Running the Application

1. Start the first peer (node) by running:

    ```bash
    cargo run
    ```

2. Open another terminal and run the second instance of the application (simulating another peer):

    ```bash
    cargo run
    ```

3. In each terminal, enter messages via `STDIN`. Messages sent from one peer will be delivered to all connected peers subscribed to the same Gossipsub topic.

4. You'll see the output of messages received from other peers displayed in each terminal.

## Example Output

### Peer 1:

```bash
Local node is listening on /ip4/192.168.1.2/udp/52324/quic-v1
Enter messages via STDIN and they will be sent to connected peers using Gossipsub
mDNS discovered a new peer: PeerId("12D3KooWRo...")
Added explicit peer: PeerId("12D3KooWRo...")
Got message: 'Hello from Peer 2' with id: MessageId("msg-id-1") from peer: PeerId("12D3KooWRo...")

