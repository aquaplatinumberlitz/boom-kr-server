"""Explicit lifetime/rate budgets for the fixed sixteen-transport diagnostic.

These emulator limits are not a recovered KR protocol law. Restart replaces a
transport lifetime; a Battle reset does not erase resource use or refill a burst.
"""


class FourInputBudget:
    def __init__(self, *, input_bytes=1024*1024, input_chunks=4096,
                 output_bytes=2*1024*1024, output_calls=4096, rate=32, burst=128):
        values=(input_bytes,input_chunks,output_bytes,output_calls,rate,burst)
        if any(type(x) is not int or x<1 for x in values):
            raise ValueError('positive integer diagnostic budgets required')
        self.limits=dict(input_bytes=input_bytes,input_chunks=input_chunks,
                         output_bytes=output_bytes,output_calls=output_calls,
                         rate=rate,burst=burst)
        self.records={}

    def _state(self, ordinal):
        return self.records.get(ordinal,dict(input_bytes=0,input_chunks=0,
                                           output_bytes=0,output_calls=0))

    def _token(self, state, kind, now):
        previous=state.get(kind+'_clock',now)
        # A regressing observation cannot mint tokens or move the refill origin.
        tokens=min(self.limits['burst'],state.get(kind+'_tokens',self.limits['burst'])
                   +max(0,now-previous)*self.limits['rate'])
        return tokens,max(now,previous)

    def chunk(self, ordinal, size, now):
        state=self._state(ordinal)
        if state['input_bytes']+size>self.limits['input_bytes']:
            return 'input_lifetime_bytes'
        if state['input_chunks']>=self.limits['input_chunks']:
            return 'input_lifetime_chunks'
        tokens,clock=self._token(state,'chunk',now)
        if tokens<1:
            return 'input_chunk_rate'
        state.update(input_bytes=state['input_bytes']+size,input_chunks=state['input_chunks']+1,
                     chunk_tokens=tokens-1,chunk_clock=clock)
        self.records[ordinal]=state
        return None

    def frame(self, ordinal, now):
        state=self._state(ordinal);tokens,clock=self._token(state,'frame',now)
        if tokens<1:
            return 'input_frame_rate'
        state.update(frame_tokens=tokens-1,frame_clock=clock)
        self.records[ordinal]=state
        return None

    def sending(self, ordinal, size):
        state=self._state(ordinal)
        if state['output_bytes']+size>self.limits['output_bytes']:
            return 'output_lifetime_bytes'
        if state['output_calls']>=self.limits['output_calls']:
            return 'output_lifetime_calls'
        # Reserve the attempted call. A failed/partial kernel write must not
        # refund bytes that may already be observable to the recipient.
        state.update(output_bytes=state['output_bytes']+size,output_calls=state['output_calls']+1)
        self.records[ordinal]=state
        return None
