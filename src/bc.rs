use crate::value::{Value, Verb as V};
use crate::parse::{self, Node};

#[repr(u8)]
#[derive(Debug)]
pub enum Op {
    Trap=0,             // crash and burn
    Return,             // return
    TraceX, TraceY,     //  \x        \y
    XZ, YZ,             // x:()      y:()
    XK(u8), YK(u8),     // x:K@?     y:K@?
    XY, YX,             // x:y       y:x
    XV(u8), YV(u8),     // x:V@?     y:V@?
    VX(u8), VY(u8),     // V[?]:x    V[?]:y
    XS, YS,             // x:pop     y:pop
    SX, SY,             // push x    push y
    XaY, YaX,           // x:x@y     x:y@x
    Da(V), DaKX(V, u8), // x:V[x;y]  x:V[K[?];x]
    MaX(V), MaY(V),     // x:(V:)[x] x:(V:)[y]
    Swap,               // (x;y):(y;x)
    // TODO: proj
}

#[derive(Debug)]
pub struct Fun {
    args: Vec<String>,
}

struct CompileState {
    args: Vec<String>,
    args_defined: bool,
    consts: Vec<Value>,
    vars: Vec<String>,
    bc: Vec<Op>,
}

// {x:1}
impl CompileState {
    fn find_vars(&mut self, node: &Node) -> Option<()> {
        match node {
            // we can error on non-existant vars in compile
            Node::Var(name) => {
                if !self.args_defined && name.len() == 1 {
                    let min = match name[0] {
                        'x' => 1, 'y' => 2, 'z' => 3, _ => 0
                    };
                    if self.args.len() < min {
                        self.args = match min {
                            1 => vec!['x'.into()],
                            2 => vec!['x'.into(), 'y'.into()],
                            3 => vec!['x'.into(), 'y'.into(), 'z'.into()],
                            _ => unreachable!()
                        }
                    }
                }
            },
            Node::Assign(name, node) => {
                self.find_vars(node);
                let name: String = name.iter().collect();
                if !self.vars.contains(&name) {
                    self.vars.push(name);
                }
            },
            Node::List(_) => todo!(),
            Node::Progn(_) => todo!(),
            Node::Apply(_, _) => todo!(),
            Node::Apply2(_, _, _) => todo!(),
            Node::WithL { l, f } => todo!(),
            Node::ApplyA { v, a } => todo!(),
            Node::ApplyN(_, _) => todo!(),
            Node::Compose(_, _) => todo!(),
            Node::Noun(node) => self.find_vars(node)?,
            _ => {},
        };
        Some(())
    }

    fn expr(&mut self, node: Node) -> Result<(), String> {
        match node {
            Node::Literal(val) => {
                let i = self.consts.len() as u8;
                self.consts.push(val);
                self.bc.push(Op::XK(i))
            },
            Node::Var(_) => todo!(),
            Node::Assign(_, _) => todo!(),
            Node::Verb(_) => todo!(),
            Node::Monad(_) => todo!(),
            Node::Adverb(_) => todo!(),
            Node::List(_) => todo!(),
            Node::Fun(_) => todo!(),
            Node::Progn(_) => todo!(),
            Node::Apply(_, _) => todo!(),
            Node::Apply2(_, _, _) => todo!(),
            Node::WithL { l: _, f: _ } => todo!(),
            Node::ApplyA { v: _, a: _ } => todo!(),
            Node::ApplyN(_, _) => todo!(),
            Node::Compose(_, _) => todo!(),
            Node::Noun(_) => todo!(),
        };
        Ok(())
    }

    fn compile(node: Node) -> Result<Fun, String> {
        let mut state = CompileState {
            args: Vec::new(),
            args_defined: false,
            consts: Vec::new(),
            vars: Vec::new(),
            bc: Vec::new(),
        };
        state.find_vars(&node);
        state.expr(node)?;
        Ok(Fun {
            args: state.args
        })
    }
}

pub fn compile(node: Node) -> Result<Fun, String> {
    CompileState::compile(node)
}
