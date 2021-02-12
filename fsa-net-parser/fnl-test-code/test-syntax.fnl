 network Test {

    //this is a single line comment
    events ev, ev2
    obs olbl1,  olbls2

    automata A {
        begin s1
        state s2
        state s3

        trans t1 s1 s2 
        trans t3 s2 s3 

    }
    automata B {
        begin s1
        state s2 
        state s3

        trans t1 {
            src s1
            dst s2
            input ev(L1)
            obs olbl1
        }

    }
    link L1 A B 
    link L2 A B
    events a, b, c
}

//comment outside blocks
/*
    This is a multi line comment
*/
request Test {
    space 
}

network "Weird Long Name" {

    automata A {
        begin "State1"
    }

    automata 'Test Name' {
        begin S1
    }

    link L2 'Test Name' A
} 

/***
    multi line can contain a lot of * and /
*/

