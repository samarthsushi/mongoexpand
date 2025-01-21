# mongoexpand
write expansions(or macros) for mongoql.

e.g.
```
count { 
        { $field }
        {
            { $group: {
                _id: '$field',
                cnt: { $sum: 1 }
            }
        }
    }
```
this is a macro input you will provide, which has some arguments and the expansion it will grow to.
and this is how you would use it in your queries:
```
$count: { $branch }
```
this will expand to:
```
$count:{$group:{_id:'$branch',cnt:{$sum:1}}}
```