use oorandom::{Rand32, Rand64};
fn main() {
 let args: Vec<_> = std::env::args().collect();
 let width: u64 = args[2].parse().unwrap();
 if args[1] == "32" {
  let mut r = Rand32::from_state((0, 0));
  eprintln!("before width={} state={:?}", width, r.state());
  let out = r.rand_range(0..width as u32);
  println!("returned={} state={:?}", out, r.state());
 } else {
  let mut r = Rand64::from_state((0, 0));
  eprintln!("before width={} state={:?}", width, r.state());
  let out = r.rand_range(0..width);
  println!("returned={} state={:?}", out, r.state());
 }
}
