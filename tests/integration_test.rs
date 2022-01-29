mod common;
use common::{assert_expression, assert_script_output, assert_script_error};

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

#[test]
fn ifs() {
    assert_script_output("var a = 1; print a;", "1");
    assert_script_output("if(true) { print \"true\"; } print 2;", "true\n2");
    assert_script_output(
        "var a; if(1 == 2) { a = \"true\"; } else { a = \"false\"; } print a;",
        "false",
    );
}

#[test]
fn loops() {
    assert_script_output(
        "var a = 0; while(a < 5) { print a; a = a + 1; }",
        "0\n1\n2\n3\n4",
    );
    assert_script_output(
        "for(var i = 0; i < 4; i = i + 1) { print i; }",
        "0\n1\n2\n3",
    );
}



#[test]
fn runtime_errors() {
    assert_script_error(
        "var b; print a;",
        "Undefined variable 'a'",
    );
}




#[test]
fn procedures() {
    assert_script_output(
        "fun pepe() { print 1; } print pepe;",
        "<fn 'pepe'>",
    );
    assert_script_output(
        "fun pepe() { print 1; } pepe();",
        "1",
    );
    assert_script_output(
        "fun pepe() { print 1; } print pepe();",
        "1\nnil",
    );
}

#[test]
fn functions() {
    assert_script_output(
        "fun add(a, b) { return a + b; } print add;",
        "<fn 'add'>",
    );
    assert_script_output(
        "fun add(a, b) { return a + b; } print add(2,3);",
        "5",
    );
}


#[test]
fn recursive_functions() {
    assert_script_output(
        "
fun fact(n) {
    if(n <= 1) { 
        return 1; 
    } else { 
        return n * fact(n-1); 
    } 
} 

print fact(5);",
        "120",
    );
    assert_script_output(
        "
fun fib(n) {
  if (n < 2) return n;
  return fib(n - 2) + fib(n - 1);
}

print fib(6);",
        "8",
    );
}



#[test]
fn native_functions() {
    assert_script_output(
        "print clock;",
        "<native 'clock'>",
    );
    // assert_script_output(
    //     "print clock();",
    //     "",
    // );

/*
fun fib(n) {
  if (n < 2) return n;
  return fib(n - 2) + fib(n - 1);
}

print fib(5);
var start = clock();
print clock() - start;
 */
}
