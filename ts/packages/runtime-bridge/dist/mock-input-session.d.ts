import type { InputActionReplayReceipt, InputContextChangeReceipt, InputContextCommand, InputContextStackState, InputResolutionReceipt, InputSessionConfigureRequest, InputSessionSnapshot, RawInputSample, RecordedInputAction } from '@asha/contracts';
export declare class MockInputSession {
    #private;
    initialize(): void;
    configure(request: InputSessionConfigureRequest): InputSessionSnapshot;
    applyContextCommand(command: InputContextCommand): InputContextChangeReceipt;
    resolve(sample: RawInputSample): InputResolutionReceipt;
    readContextState(): InputContextStackState;
    replay(record: RecordedInputAction): InputActionReplayReceipt;
}
//# sourceMappingURL=mock-input-session.d.ts.map