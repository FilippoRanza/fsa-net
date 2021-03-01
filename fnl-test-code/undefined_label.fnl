



network NetTest {
    rel r1, r2, r3

    automata A {
        begin s0
        state s1
        state s2

        trans t0 s0 s2
        trans t1 s1 s2
        trans t2 s2 s1
        trans t3 s1 s0
    }

}

request NetTest {
    diagnosis r1, r2, r4
}
