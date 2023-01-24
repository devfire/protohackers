use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use std::collections::BTreeMap;

// https://doc.rust-lang.org/std/primitive.i32.html#method.from_be_bytes
fn read_be_i32(input: &[u8]) -> i32 {
    i32::from_be_bytes(input[..4].try_into().unwrap())
}

fn calculate_average(btree_db: &BTreeMap<i32, i32>, start: &i32, end: &i32) -> i32 {
    use std::ops::Bound::Included;
    let mut total = 0; // sum of prices
    let mut final_count = 0; // number of entries

    println!("Calculating average between {} and {}", start, end);

    // https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html#method.range
    for (count, (key, value)) in btree_db.range((Included(start), Included(end))).enumerate() {
        println!("{key}: {value}");
        total += value;
        final_count = count;
    }
    // average is total / count
    println!("Total: {} count: {}", total, final_count);

    total / final_count as i32
}

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    loop {
        // The second item contains the ip and port of the new connection.
        let (stream, addr) = listener.accept().await.unwrap();

        println!("New connection from {}", addr);

        // A new task is spawned for each inbound socket.  The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

async fn process(stream: TcpStream) {
    let (mut reader, mut writer) = tokio::io::split(stream);

    // A hashmap is used to store data
    let mut db: BTreeMap<i32, i32> = BTreeMap::new();

    // loop {
    //declare a buffer precisely 9 bytes long
    let mut buffer = [0; 9];

    let n = reader
        .read_exact(&mut buffer)
        .await
        .expect("Unable to read from buffer");

    // if n == 0 {
    //     break;
    // }

    let msg_type = &buffer[0];

    // Read slices 1,2,3,4.
    // Since we have a slice rather than an array, fallible conversion APIs can be used
    let first_half_decoded = read_be_i32(&buffer[1..=4]);
    let second_half_decoded = read_be_i32(&buffer[5..n]);

    println!(
        "Type: {}, first: {}, second: {}",
        msg_type, first_half_decoded, second_half_decoded
    );

    match msg_type {
        73 => {
            db.insert(first_half_decoded, second_half_decoded);
        }
        81 => {
            let avg = calculate_average(&db, &first_half_decoded, &second_half_decoded);
            writer
                .write_all(avg.to_string().as_bytes())
                .await
                .expect("Writing avg failed");

            writer
                .write_all(b"\n")
                .await
                .expect("Failed ending buffer write");
        }
        _ => panic!("unknown msg type"),
        // }
    }
}
