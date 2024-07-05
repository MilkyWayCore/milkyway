# MilkyWay
MilkyWay is utility for manging servers not through SSH, but through sending commands to groups of servers. To add an extra security it is suggested to use post-quantum cryptography for verifying that message originates from trusted source.

## Whole-project documentation
The diagram on how current vision of architecture looks like is located in [doc/](doc/) folder.

# Broker
The broker (would be) implemented in milkywaysrvd. It advertises itself on the network enabling peers to communicate with each one.

# Peers
Peer software (would be) implemented in milkywayd. The peers send status messages. Each peer have own certificate(which must be provided during peer upbringning either automatically or manually) and signs all messages sent.

# Modules
While module are not yet implemented, general idea that they would be able to execute on both broker and peers to perform customized action(e.g. bringing up VPN, updating software, etc.). It is planned to do native modules in Rust and add easier way to write them in Ruby.
Also digital signature for modules should be verified to avoid attacks with tampering modules.

# CLI
It is intended that only CLI would be able to sign commands with proper certificate which makes it impossible to execute malicious command for somebody who has no certificate(equivalently access to local computer)

## Example
### VPN setup
In perfect future we would be able to do something like this:
```bash
mway group --make-peer-group --all-unassigned --name my_network # Add all new peers to new group
mway update --group my_network --no-reboot # Install system updates on all peers in network, but do not reboot them
mway vpn --initialize-wireguard-on 10.10.0.1 --new-peer-group --peer-group-name virt_net # Install and set-up wireguard on peer with IP 10.10.0.1 and create peer group for its client
mway vpn --add-peers 172.16.42.0/24 --all --peer-group virt_net # Add local peers to unite them in one virtual network
mway reboot --peer-group virt_net # Reboot all peers in virt_net group
```

(Or maybe even somehow better)

### New virtual machine
```bash
mway virt --host 10.10.0.2 --new-machine --os-image milkywaylinux --static-ip 10.10.0.42 --wait --peer-group isolated_docker_compose_app # Set up new virtual machine in peer group isolated_docker_compose_app with static ip 10.10.0.42 and block CLI until machine goes up
mway install --peer 10.10.0.42 --reboot docker # Install docker and reboot
mway service --peer 10.10.0.42 enable docker # Enable docker(actually if it is server it should be enabled by default)
mway secrets --values "MY_VERY_SECRET_VARIABLE=very_secret_value" --peer 10.10.0.42 --secret-group my_app # Push secret variables to peer and put them in secret group 
mway docker start --compose docker-compose.yml --enable # Start docker-compose app and enable its auto start
```

### Peer setup
As this program is obviously wouldn't be shipped by anybody(except maybe AUR and GitHub), the peer setup is planned to do in two ways:
* Custom images with milkyway pre-installed(with special certificate partition, so we would securely deliver them to target peers)
* cloud-init(less secure as we would need to trust broker who advertises itself or trust cloud-init itself)

# Project structure
> **NOTE:** Currently only implemented parts described here

## libmilkyway
Contains common structures and methods for broker, peer and CLI. Currently libmilkyway implements:
* Serialization and deserialization of Rust structs to byte arrays
* Post-quantum PKI

## libmilkyway\_derive
Library with procedural macros for using `#[derive]`, does nothing special, event tested in libmilkyway itself
