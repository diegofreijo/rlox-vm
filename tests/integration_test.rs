use rlox_vm::{value::Value, compiler::Compiler, vm::{VM, InterpretResult}};


#[test]
fn expresions() {
	assert_expression("1", Value::Number(1.0));
	assert_expression("1+2", Value::Number(3.0));
	assert_expression("!nil", Value::Boolean(true));
	assert_expression("!(5 - 4 > 3 * 2 == !nil)", Value::Boolean(true));
}


fn assert_expression(source: &str, expected: Value) {
	let source2= String::from(source);
    let mut compiler = Compiler::from(&source2);
	compiler.compile();

	assert!(!compiler.had_error);

	let mut vm = VM::new();
	let result = vm.run(&compiler.chunk);
	assert_eq!(result, InterpretResult::Ok(expected));
}
