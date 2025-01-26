use mongoexpand::{ExpansionEngine};

fn main() {
    let s = "$count { 
        { $field, $a }
        {
            { $group: {
                _id: '$field',
                $a: { $sum: 1 }
            }
        }
    }";
    let q = "$count: { $branch }";
    let mut me = ExpansionEngine::new();
    let ret = me.add_expansion(&s);
    println!("add_expansion -> {:?}\n{:?}", ret, me);
}
