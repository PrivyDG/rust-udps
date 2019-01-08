# rust-udps

`rust-udps` is both a crate and system library implementing my custom "secure" protocol
_UDPS_ on top of _UDP_, natively in Rust.

## Basic connection scheme

_Actor: Method(Data) [-> return data variable]_

* Client: Connect() -> conn_id
* Server: Ack(conn_id)
* Client: PublicKey(cl_pubkey) -> cl_pubkey_id
* Server: Ack(cl_pubkey_id)
* Server: PublicKey(sv_pubkey) -> sv_pubkey_id
* Client: Ack(sv_pubkey_id)
* Client: "GENERATE SECRET KEY" -> seckey
* Client: SecretKey(seckey) -> seckey_id
* Server: Ack(seckey_id)

**-- CONNECTED**

## Basic steps

The public key algorithm used will be RSA-2048 for now,
and the secret key algorithm some form of AES.

The connect step will initiate a connection between two UDPS endpoints. The client will then send its public key until the server either timeouts or acknowledges the public key.
The server will then do the same.

After the public keys have been exchanged, the endpoint that initially send the connection request will generate a secret key and initialization vector, encrypt it with the other endpoints public key and send it over. After the other endpoint acknowledges this secret key, all future `Data` and `DataSeq` packages will be encrypted using this secret key.


## A note on security

`rust-udps` is probably not very secure. **USE AT YOUR OWN RISK!!!**



