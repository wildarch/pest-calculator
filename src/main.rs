
use std::io::{self, BufRead};
use pest::iterators::{Pair, Pairs};
use pest::prec_climber::PrecClimber;
use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "calculator.pest"]
pub struct CalculatorParser;

lazy_static::lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = {
        use pest::prec_climber::{Assoc::*, Operator};
        use Rule::*;

        PrecClimber::new(vec![
            Operator::new(add, Left) | Operator::new(subtract, Left),
            Operator::new(multiply, Left) | Operator::new(divide, Left) | Operator::new(modulo, Left),
        ])
    };
}

#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    UnaryMinus(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PREC_CLIMBER.climb(
        pairs,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::integer => Expr::Integer(pair.as_str().parse::<i32>().unwrap()),
            // expression in parentheses.
            Rule::expr => parse_expr(pair.into_inner()),
            Rule::unary_minus => Expr::UnaryMinus(Box::new(parse_expr(pair.into_inner()))),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
        },
        |lhs: Expr, op: Pair<Rule>, rhs: Expr| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                Rule::modulo => Op::Modulo,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
}

#[derive(Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo
}

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
