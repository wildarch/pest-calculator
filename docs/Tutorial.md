# Precedence climbing with pest - calculator example

## Calculator example
We want our calculator to be able to parse simple equations that consist of integers and simple binary operators.
We want to support parenthesis and unary minus.
For example:
```
1 + 2 * 3
-(2 + 5) * 16
```

## Grammar
We start by defining our atoms, bits of self-contained syntax that cannot be split up into smaller parts.
For our calculator we start with just simple integers:
```pest
// No whitespace allowed between digits
integer = @{ ASCII_DIGIT+ }

atom = _{ integer }
```

Next, our binary operators:
```pest
bin_op = _{ add | subtract | multiply | divide }
	add = { "+" }
	subtract = { "-" }
	multiply = { "*" }
	divide = { "/" }
```

These two rules will be the input to the
[`PrecClimber`](https://docs.rs/pest/latest/pest/prec_climber/struct.PrecClimber.html). 
It expects to receive atoms separated by operators, like so: `atom, bin_op, atom, bin_op, atom, ...`.

Corresponding to this format, we define our rule for expressions:
```pest
expr = { atom ~ (bin_op ~ atom)* }
```
This defines the grammar which generates the required input for the precedence climber.

## Abstract Syntax Tree
We want to convert our input into an abstract syntax tree.
For this we define the following types:

```rust
#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}
```

Note the `Box<Expr>` required because Rust 
[does not allow unboxed recursive types](https://doc.rust-lang.org/book/ch15-01-box.html#enabling-recursive-types-with-boxes). 

There is no separate atom type, any atom is also a valid expression.

## Precedence climber
The precedence of operations is defined in the precedence climber.

An easy approach is to define the precedence climber as global using [`lazy_static`](https://docs.rs/lazy_static/1.4.0/lazy_static/).

Adhering to standard rules of arithmetic, 
we will define addition and subtraction to have lower priority than multiplication and division, 
and make all operators left associative.

```rust
lazy_static::lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::{Assoc::*, Operator};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrecClimber::new(vec![
            // Addition and subtract have equal precedence
            Operator::new(add, Left) | Operator::new(subtract, Left),   
            Operator::new(multiply, Left) | Operator::new(divide, Left),
        ])
    };
}
```

We are almost there, the only thing that's left is to use our precedence climber.
For this the `climb` function is used, it takes a vector of pairs and two functions.
One function is executed for every primary (atom), and the infix function is executed for every binop with its new left 
hand and right hand side according to the precedence rules defined earlier.
In this example we create an AST in the precedence climber.

```rust
pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PREC_CLIMBER.climb(
        pairs,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::integer => Expr::Integer(pair.as_str().parse::<i32>().unwrap()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
        },
        |lhs: Expr, op: Pair<Rule>, rhs: Expr| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
}
```

Here's an example of how to use the parser.

```rust
fn main() -> io::Result<()> {
    for line in io::stdin().lock().lines() {
        match CalculatorParser::parse(Rule::equation, &line?) {
            Ok(pairs) =>{
                println!("Parsed: {:#?}", parse_expr(pairs));
            }
            Err(e) => {
                eprintln!("Parse failed: {:?}", e);
            }
        }
    }
    Ok(())
}
```

With this we can parse the following simple equation:
```
> 1 * 2 + 3 / 4
Parsed: BinOp {
    lhs: BinOp {
        lhs: Integer( 1 ),
        op: Multiply,
        rhs: Integer( 2 ),
    },
    op: Add,
    rhs: BinOp {
        lhs: Integer( 3 ),
        op: Divide,
        rhs: Integer( 4 ),
    },
}
```