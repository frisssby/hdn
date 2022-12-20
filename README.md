# hdn

Hash delivery network is a distributed key-value storage, which is built as a TCP network that synchronizes data among the nodes and supports storing hashes in a remote storage and loading them from it.

## Config

Server requires a configuration file which is supposed to have the following structure:

```json
{
    "user": "Elina Safarova",
    "nodes": [
        "127.0.0.1",
        "127.0.0.2",
        "127.0.0.3"
    ],
    "client_port": 43000,
    "peer_port": 42000
}
```

## Launching

```bash
git clone git@github.com:frisssby/hdn.git
cd hdn
cargo run -- --config <PATH> --id <NODE_ID>
```

**PATH** is the location of the configuration file.

**NODE_ID** is the 0-based index of the node in the configuration file.

## Simulation of a network with three servers

The repository also provides a simulation of the three-node *hdn* storage working in a docker container.

To launch it, run

```bash
cd simulation
docker-compose up
```

This command sets up three servers responsible for three different geolocations. They will be listening for TCP connections on `localhost:4300,4301,4302`.

## Protocol

Communication with the server occurs through **json** messages.

After the successful connection, the client receives a greeting message:

```json
  {
    "student_name" : "Elina Safarova"
  }
```

There are two types of valid requests:

+ **Store** a *hash* under a *key*

    ```json
    {
      "request_type": "store",
      "key": "some_key",
      "hash": "0b672dd94fd3da6a8d404b66ee3f0c8"
    }
    ```

    In case the request went successful, the server responds the client with:

     ```json
    {
      "response_status": "success"
    }
    ```

+ **Load** the previously saved hash by a key

    ```json
    {
      "request_type": "load",
      "key": "some_key"
    }
    ```

    In case the request went successful, the server responds the client with:

    ```json
    {
      "response_status": "success",
      "requested_key": "some_key",
      "requested_hash": "0b672dd94fd3da6a8d404b66ee3f0c83"
    }
    ```

    If the key is not in the storage, the server will send:

    ```json
    {
      "response_status": "key not found"
    }
    ```

+ If the server gets an invalid request, it will reply with a message:

    ```json
    {
      "response_status": "invalid request"
    }
    ```
