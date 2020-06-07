import CounterRs "canister:inter_rs";

actor Counter {
    public func inc() : async () {
        await CounterRs.inc()
    };

    public func read() : async Nat {
        await CounterRs.read()
    };

    public func write(n: Nat) : async () {
        await CounterRs.write(n)
    };
}
