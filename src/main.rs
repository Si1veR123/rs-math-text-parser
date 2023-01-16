use math_parser::equation::*;

fn main() {
    let equation = EquationValue::Equation(
        Box::new(EquationOperation::Mul(
            EquationValue::Equation(
                Box::new(
                    EquationOperation::Function(
                        FunctionType::Cos,
                        EquationValue::Variable
                    )
                )
            ),
            EquationValue::Constant(15.0)
        ))
    );
    println!("{}", equation.evaluate(Some(3.14)));

    let eq_str = "(80+x)^sin(x^2+(2*5))";
    let eq = EquationValue::from(eq_str);
    println!("{:?}", eq);
    println!("{:?}", eq.evaluate(Some(10.0)));
}
