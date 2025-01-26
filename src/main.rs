use mongoexpand::{MacroEngine};

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
    let mut me = MacroEngine::new();
    let ret = me.add_macro(&s);
    println!("add_macro -> {:?}\n{:?}", ret, me);
}
