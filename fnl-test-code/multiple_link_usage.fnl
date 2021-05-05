
network TestNetwork {

    link L1 A B
    link L2 B A

    events e1, e2, e3

    automata A {
        begin a1
        state a2
        state a3

        trans t1 {
            src a1
            dst a2
            output e1(L1)
        }
        trans t2 a2 a3
        trans t3 a3 a1
    }

    automata B {
        begin b1
        state b2
        state b3

        trans t1 b1 b2
        trans t2 {
            src b2
            dst b3
            // link L2 is used two times
            output e2(L2), e2(L2) 
        }
        trans t3 b3 b1
    }

}

request TestNetwork {
    space
}
