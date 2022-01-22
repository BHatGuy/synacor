from functools import cache

# 1545: check func

# 1569: noop
# 156a: noop
# 156b: set  8000    4
# 156e: set  8001    1
# 1571: call 178b
# 1573: eq   8001 8000    6
# 1577: jf   8001 15cb






def f_orig(a, b):
    assert a < 0x8000 and b < 0x8000
    assert a >= 0 and b >= 0
    if a == 0:
        return (b + 1) % 0x8000
    else:
        if b == 0:
            a = (a + 0x7fff) % 0x8000
            b = r7
            return f_orig(a, b)
        else:
            b = (b + 0x7fff) % 0x8000
            b = f_orig(a, b)
            a = (a + 0x7fff) % 0x8000
            return f_orig(a, b)


def f_minus(a, b):
    assert a < 0x8000 and b < 0x8000
    assert a >= 0 and b >= 0
    
    if a == 0:
        return b + 1
    else:
        if b == 0:
            return f_minus(a-1, r7)
        else:
            return f_minus(a-1, f_minus(a, b-1))
          



r7 = 1
r0 = 4
r1 = 1
print(f_minus(r0, r1, r7))