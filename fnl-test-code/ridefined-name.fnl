network Test {

    automata A {
        begin s1
        state s2
        trans t1 s1 s2
    }

    automata B {
        begin s2
        state s3
        trans t1 s2 s3
    }

    obs s1

}