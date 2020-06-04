let console = {};
console.log = function () {
    // Unfortunately, Deno.core.print only takes
    // one string argument. So I have to loop through
    // to implement a var-arg console.log
    for (var i = 0; i < arguments.length; i++) {
        if (i !== 0) {
            Deno.core.print(" ");
        }
        Deno.core.print(arguments[i]);
    }
    Deno.core.print("\n")
};

var promise1 = Promise.resolve(3);
var promise2 = 42;
var promise3 = new Promise(function(resolve, reject) {
    resolve();
    //setTimeout(resolve, 100, 'foo');
});
console.log("Hello");
Promise.all([promise1, promise2, promise3]).then(function(values) {
    console.log("Hello2");
    console.log("All values are >");
    console.log(values.toString());
});

promise1.then(function (value) {
    console.log("Hello3");
    console.log("This is >");
    console.log(value);
})

/* does not work - syntax error
promise2.then(function (value) {
    console.log("Hello4");
    console.log("This is ", value);
})
*/

promise3.then(function (value) {
    console.log("Hello4");
    console.log("This is >");
    console.log(value);
})
