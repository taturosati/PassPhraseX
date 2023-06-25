pub mod dist;

use dist::PasswordDist;
use rand::distributions::DistString;

pub fn generate_password(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let mut password = String::new();
    PasswordDist.append_string(&mut rng, &mut password, length);
    password
}
