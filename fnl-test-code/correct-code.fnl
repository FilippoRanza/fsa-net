
network TestNetwork {

    link L1 A B
    link L2 B A

    events e1, e2, e3, e4

    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }

    automata B {
        begin b1
        state b2
        state b3

        trans t1 {
            src b1
            dst b2
            output e1(L2)
        }
        trans t2 {
            src b2
            dst b3
            output e1(L2)
        }
        trans t3 b3 b1
    }

}

request TestNetwork {
    space
}


network OtherNetwork {
    automata A {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }
    
    link L1 A B

    automata B {
        begin a1
        state a2
        state a3

        trans t1 a1 a2
        trans t2 a2 a3
        trans t3 a3 a1
    }

    link L2 B A
}

request OtherNetwork {
    space
}



