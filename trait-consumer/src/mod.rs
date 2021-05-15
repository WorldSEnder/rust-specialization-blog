use trait_host::ShowDetails;
use trait_impl::FizzBuzzer;

struct Context { s: FizzBuzzer }

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        <FizzBuzzer as ShowDetails>::fmt_details(&self.s, f)
    }
}

pub fn main() {
    let ctx = Context {
        s: FizzBuzzer::new(),
    };
    println!("{:?}", ctx)
}
