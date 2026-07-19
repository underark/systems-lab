1) length is the exact number of initialized elements owned by the vector
1a) length can never be greater than capacity
2) capacity is the number of elements of type 'T' that it is able to write to the allocated buffer
2a) capacity * size of type 'T' is the number of bytes available to the vector buffer for writing elements of type 'T'
2b) vector has number of elements at absolute most isize::MAX / size of 'T'
3) elements 0..length (exclusive at top end; 0 to n-1) are 'live' initialized elements owned by the vector
3a) elements length..capacity (exclusive at top end; n to capacity-1) are considered uninitialized regardless of the 'real' bytes stored there 

open to change:
1a) logically i want this to be true but it may not be depending on ZST stuff later

meta ideas:
- an invariant is allowed to be true during the execution of a method but it must hold true at both the beginning and end (public and private state)

