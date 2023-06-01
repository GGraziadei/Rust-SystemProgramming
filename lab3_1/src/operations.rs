#[derive(Debug, Clone, Copy)]
pub enum OP {
    SOMMA,
    DIFFERENZA,
    PRODOTTO,
    DIVISIONE
}

#[derive(Debug, Clone)]
pub struct OpError;


    impl OP {
        pub fn value(&self ) -> &str{
            match self {
                SOMMA => { "+" } ,
                DIFFERENZA => { "-" },
                PRODOTTO => { "*" },
                DIVISIONE => { "/" }
            }
        }

        pub fn op(&self, a : &i32, b : &i32 ) -> Result<i32, OpError> {
            Ok( match self {
                OP::SOMMA => { a+b } ,
                OP::DIFFERENZA => { a-b },
                OP::PRODOTTO => { a*b },
                OP::DIVISIONE => { a/b  }
            })
        }
    }

