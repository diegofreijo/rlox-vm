mod common;

use rlox_vm::{compiler::Compiler, vm::{VM, InterpretResult}};


#[test]
fn expresions() {
	assert_expression("1", "1");
	assert_expression("1+2", "3");
	assert_expression("!nil", "true");
	assert_expression("!(5 - 4 > 3 * 2 == !nil)", "true");
	assert_expression("\"asd\" +\"qwe \"", "asdqwe ");
	assert_expression("\"asd\" == \"asd\"", "true");
	assert_expression("\"asd\" != \"asd\"", "false");
}


fn assert_expression(exp_source: &str, expected: &str) {
	let source= format!("print {};", exp_source);
    let mut compiler = Compiler::from(&source);
	compiler.compile();

	assert!(!compiler.had_error);

	let mut vm = VM::new();
	let mut stdout = common::Output::new();
	
	let result = vm.run(&compiler.chunk, &mut stdout);

	assert_eq!(result, InterpretResult::Ok);
	assert_eq!(stdout.contents.trim_end_matches("\n"), expected);
}
