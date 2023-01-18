use std::mem;
use std::collections::BTreeMap;
use std::str::FromStr;
use strum_macros::EnumString;

pub trait EquationEval {
    fn evaluate(&self, variable: Option<f64>) -> f64;
}

#[derive(Debug)]
pub enum EquationValue {
    Constant(f64),
    Variable,
    Equation(Box<EquationOperation>)
}

impl EquationEval for EquationValue {
    fn evaluate(&self, variable: Option<f64>) -> f64 {
        match &self {
            &Self::Constant(c) => c.clone(),
            &Self::Equation(eq) => eq.evaluate(variable),
            &Self::Variable => variable.expect("Expected variable but not given one")
        }
    }
}

#[derive(Debug)]
enum ParsedSymbol<'a> {
    Constant(f64),
    Variable,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Function(FunctionType),
    Nested(&'a str),
    Processed(usize) // when a symbol has been processed, it is replaced with this, where usize is an index for a vec containing processed equations
}

impl<'a> TryFrom<&ParsedSymbol<'a>> for EquationValue {
    type Error = ();
    fn try_from(value: &ParsedSymbol<'a>) -> Result<Self, Self::Error> {
        match value {
            ParsedSymbol::Constant(f) => Ok(EquationValue::Constant(*f)),
            ParsedSymbol::Variable => Ok(EquationValue::Variable),
            ParsedSymbol::Nested(n) => Ok(EquationValue::from(*n)),
            _ => Err(())
        }
    }
}

macro_rules! filter_symbol_enum {
    ($i:ident,$type:pat_param) => {
        $i.iter()
        .enumerate()
        .filter(|(_, x)| matches!(x, $type))
        .map(|(i, _)| i)
        .collect::<Vec<usize>>()
    };
}

macro_rules! two_operand_operation_process {
    ($all_symbols: ident, $symbols:ident, $processed_operations:ident, $operation_id:literal) => {
        $symbols.iter().for_each(|&i| {
            let (split_0, split_1) = $all_symbols.split_at_mut(i);
            let (secondary_split_0, secondary_split_1) = split_1.split_at_mut(1);
            let x = secondary_split_0.first_mut().unwrap();
            let operand1_symbol = split_0.last_mut().expect("Operation has no operand");
            let operand2_symbol = secondary_split_1.first_mut().expect("Operation has no operand");

            let operand1_val;
            let operand2_val;

            if let &mut ParsedSymbol::Processed(i) = operand1_symbol {
                operand1_val = $processed_operations.remove(&i).unwrap();
            } else {
                operand1_val = (&*operand1_symbol).try_into().expect("Invalid operand");
            }

            if let &mut ParsedSymbol::Processed(i) = operand2_symbol {
                operand2_val = $processed_operations.remove(&i).unwrap();
            } else {
                operand2_val = (&*operand2_symbol).try_into().expect("Invalid operand");
            }

            mem::swap(operand1_symbol, &mut ParsedSymbol::Processed(i));
            mem::swap(operand2_symbol, &mut ParsedSymbol::Processed(i));
            mem::swap(x, &mut ParsedSymbol::Processed(i));

            let operation = EquationOperation::two_operand_op_from_int($operation_id, operand1_val, operand2_val);
            let equation_value = EquationValue::Equation(Box::new(operation));

            $processed_operations.insert(
                i,
                equation_value
            );
        });
    }
}

impl From<&str> for EquationValue {
    fn from(string: &str) -> Self {
        let mut parsed_symbols: Vec<ParsedSymbol> = vec![];
        let mut index = 0;
        loop {
            let c_option = string.chars().nth(index);
            let c = match c_option {
                Some(c) => c,
                None => break
            };

            match c {
                '(' => {
                    let start_i = index+1;
                    let end_i;
                    let mut nest_depth = 0;
                    'inner: loop {
                        index += 1;
                        match string.chars().nth(index) {
                            None => {
                                panic!("reached end of string without closing bracket")
                            },
                            Some('(') => {
                                nest_depth += 1
                            },
                            Some(')') => {
                                if nest_depth == 0 {
                                    end_i = index;
                                    break 'inner;
                                } else {
                                    nest_depth -= 1;
                                }
                            },
                            _ => ()
                        }
                    }
                    // inner string without brackets
                    parsed_symbols.push(ParsedSymbol::Nested(&string[start_i..end_i]))
                },
                '+' => {
                    parsed_symbols.push(ParsedSymbol::Add);
                },
                '-' => {
                    parsed_symbols.push(ParsedSymbol::Sub);
                },
                '*' => {
                    parsed_symbols.push(ParsedSymbol::Mul);
                },
                '/' => {
                    parsed_symbols.push(ParsedSymbol::Div);
                },
                '^' => {
                    parsed_symbols.push(ParsedSymbol::Pow);
                },
                'x' => {
                    parsed_symbols.push(ParsedSymbol::Variable)
                },
                ' ' => (),
                _ => {
                    if c.is_ascii_alphabetic() {
                        // function
                        let first_function_char_index = index;
                        'inner: loop {
                            index += 1;
                            let next_c = string.chars().nth(index).expect("shouldnt be at end of string yet");
                            if !next_c.is_ascii_alphabetic() {
                                index -= 1;
                                parsed_symbols.push(
                                    ParsedSymbol::Function(FunctionType::from_str(&string[first_function_char_index..=index]).expect("Invalid function"))
                                );
                                break 'inner;
                            }
                        }
                    }
                    else if c.is_ascii_digit() {
                        // constants
                        let first_int_char_index = index;
                        let last_int_char_index;
                        'inner: loop {
                            index += 1;
                            let next_c = string.chars().nth(index);

                            match next_c {
                                Some(c) => {
                                    if (!c.is_ascii_digit()) & (c != '.') {
                                        index -= 1;
                                        last_int_char_index = index;
                                        break 'inner;
                                    }
                                }
                                None => {
                                    index -= 1;
                                    last_int_char_index = index;
                                    break 'inner;
                                }
                            }
                        }

                        parsed_symbols.push(
                            ParsedSymbol::Constant(string[first_int_char_index..=last_int_char_index].parse::<f64>().unwrap())
                        )
                    }
                }
            }

            index += 1;
        }
        
        if parsed_symbols.len() == 1 {
            match parsed_symbols.get(0).unwrap() {
                ParsedSymbol::Constant(f) => return EquationValue::Constant(f.clone()),
                ParsedSymbol::Variable => return EquationValue::Variable,
                _ => panic!("Single symbol but not const or var")
            }
        }

        // convert symbols to equation values

        // unique index of the symbol mapped to the processed equationvalue
        let mut equation_operation_store: BTreeMap<usize, EquationValue> = BTreeMap::new();

        // functions
        filter_symbol_enum!(parsed_symbols, ParsedSymbol::Function(_))
            .iter()
            .for_each(|&i| {
                let (split_0, split_1) = parsed_symbols.split_at_mut(i+1);
                let value_symbol = split_1.first_mut().expect("Function without value");
                let x = split_0.last_mut().unwrap();

                match x {
                    ParsedSymbol::Function(f) => {
                        let value: EquationValue = (&*value_symbol).try_into().expect("Invalid value after function");
                        let function = EquationOperation::Function(f.clone(), value);

                        // replace symbols with index of processed equation
                        mem::swap(x, &mut ParsedSymbol::Processed(i));
                        mem::swap(value_symbol, &mut ParsedSymbol::Processed(i));

                        equation_operation_store.insert(
                            i,
                            EquationValue::Equation(Box::new(function))
                        );
                    },
                    _ => unreachable!()
                }
        });

        // powers
        let symbols = filter_symbol_enum!(parsed_symbols, ParsedSymbol::Pow);
        two_operand_operation_process!(parsed_symbols, symbols, equation_operation_store, 4);

        // multiply
        let symbols = filter_symbol_enum!(parsed_symbols, ParsedSymbol::Mul);
        two_operand_operation_process!(parsed_symbols, symbols, equation_operation_store, 2);

        // divide
        let symbols = filter_symbol_enum!(parsed_symbols, ParsedSymbol::Div);
        two_operand_operation_process!(parsed_symbols, symbols, equation_operation_store, 3);

        // add
        let symbols = filter_symbol_enum!(parsed_symbols, ParsedSymbol::Add);
        two_operand_operation_process!(parsed_symbols, symbols, equation_operation_store, 0);

        // sub
        let symbols = filter_symbol_enum!(parsed_symbols, ParsedSymbol::Sub);
        two_operand_operation_process!(parsed_symbols, symbols, equation_operation_store, 1);

        equation_operation_store.into_values().next().unwrap()
    }
}

impl From<String> for EquationValue {
    fn from(string: String) -> Self {
        Self::from(string.as_str())
    }
}


#[derive(Debug)]
pub enum EquationOperation {
    Add(EquationValue, EquationValue),
    Sub(EquationValue, EquationValue),
    Mul(EquationValue, EquationValue),
    Div(EquationValue, EquationValue),
    Pow(EquationValue, EquationValue),
    Function(FunctionType, EquationValue)
}

impl EquationOperation {
    fn two_operand_op_from_int(i: u8, operand1: EquationValue, operand2: EquationValue) -> Self {
        match i {
            0 => Self::Add(operand1, operand2),
            1 => Self::Sub(operand1, operand2),
            2 => Self::Mul(operand1, operand2),
            3 => Self::Div(operand1, operand2),
            4 => Self::Pow(operand1, operand2),
            _ => panic!("Not a two operand operation")
        }
    }
}

#[derive(EnumString, Debug, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum FunctionType {
    Sin,
    Cos,
    Tan,
    Sinh,
    Cosh,
    Tanh,
    Exp,
    Acos,
    Asin,
    Atan,
    Asinh,
    Acosh,
    Atanh,
    Ln
    // Root,
    // Log
}

impl EquationEval for EquationOperation {
    fn evaluate(&self, variable: Option<f64>) -> f64 {
        match &self {
            EquationOperation::Add(operand1, operand2) => operand1.evaluate(variable) + operand2.evaluate(variable),
            EquationOperation::Sub(operand1, operand2) => operand1.evaluate(variable) - operand2.evaluate(variable),
            EquationOperation::Mul(operand1, operand2) => operand1.evaluate(variable) * operand2.evaluate(variable),
            EquationOperation::Div(operand1, operand2) => operand1.evaluate(variable) / operand2.evaluate(variable),
            EquationOperation::Pow(operand1, operand2) => operand1.evaluate(variable).powf(operand2.evaluate(variable)),
            EquationOperation::Function(function, operand) => {
                let value = operand.evaluate(variable);
                match function {
                    FunctionType::Acos => value.acos(),
                    FunctionType::Sin => value.sin(),
                    FunctionType::Cos => value.cos(),
                    FunctionType::Tan => value.tan(),
                    FunctionType::Sinh => value.sinh(),
                    FunctionType::Cosh => value.cosh(),
                    FunctionType::Tanh => value.tanh(),
                    FunctionType::Exp => value.exp(),
                    FunctionType::Asin => value.asin(),
                    FunctionType::Atan => value.atan(),
                    FunctionType::Asinh => value.asinh(),
                    FunctionType::Acosh => value.cosh(),
                    FunctionType::Atanh => value.tanh(),
                    FunctionType::Ln => value.ln()
                }
            },
        }
    }
}
