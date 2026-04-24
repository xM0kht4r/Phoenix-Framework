use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn random_name() -> String {
    // Generate a random name
    let rand_name: String = thread_rng()
        .sample_iter(Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    rand_name
}