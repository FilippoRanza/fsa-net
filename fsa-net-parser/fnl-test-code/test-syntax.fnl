network Test {
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
        }

    }
    link L1 A B 
}

request {
    space Test
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



