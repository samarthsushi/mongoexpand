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
    let q = "$count:{$branch}";
    let mut me = ExpansionEngine::new();
    let ret = me.add_expansion(&s);
    println!("add expansion ::= {:?}\n{:?}", ret, me);
    let q_ret = me.query(&q);
    match q_ret {
        Ok(x) => println!("> {x}"),
        Err(e) => println!("err: {:?}", e)
    };
}
