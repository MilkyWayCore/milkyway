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

```
mway> group/create peers=* name=my-net
Group created successfully
mway> system/packages/install pkgs=wireguard group=my-net no-reboot
Installing packages
wireguard:
  - peer1...OK
  - peer2...OK
  - peer3...OK
mway> system/packages/update group=my-net no-reboot
Updating packages on:
peer1...OK
peer2...OK
peer3...OK
mway> wireguard init host=peer1
Started up WireGuard server on peer1
mway> wireguard add-peers host=peer1 peers=peer1,peer2
Added peers successfully
mway> system/reboot group=my-net
peer1 is down
peer2 is down
peer3 is down
peer1 is up
peer2 is up
peer3 is up
mway>
```

Or just write state in YAML:

```yaml
system:
  packages:
    - update
    - install:
        - wireguard
  reboot:
    - on-state-apply
wireguard:
  - init:
      host: peer1
  - add_peers:
      host: peer1
      peers:
        - peer2
        - peer3 
```

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
