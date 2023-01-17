# Usage

```rust
let eq_str = "(80+x)^sin(x^2+(2*5))";
let eq = EquationValue::from(eq_str);
println!("{:?}", eq.evaluate(Some(10.0)));

// no variable
let eq_str = "(80+5)^sin(2^2+(2*5))";
let eq = EquationValue::from(eq_str);
println!("{:?}", eq.evaluate(None));
```

# Support

## Functions
- Sin
- Cos
- Tan
- Sinh
- Cosh
- Tanh
- Exp
- Acos
- Asin
- Atan
- Asinh
- Acosh
- Atanh

## Operations
- Add (a+b)
- Subtract (a-b)
- Multiply (a*b)
- Divide (a/b)
- Power (a^b)

# Tips
Negative numbers are currently supported by subtracting from 0 e.g. (0-3) = -3
