from cistell import base, field


class ConfTest(base.ConfigBase):
    cf = field.ConfigField(0)


def test_isolation_both_instantiate_both_first() -> None:
    """test that one instance do not affect the other when both are instantiated first"""
    conf1 = ConfTest()
    conf2 = ConfTest()
    conf1.cf = 1
    assert conf1.cf == 1
    assert conf2.cf == 0


def test_isolation_both_instantiate_diff() -> None:
    """test that one instance do not affect the other when both are instantiated second"""
    conf1 = ConfTest()
    conf1.cf = 1
    assert conf1.cf == 1
    conf2 = ConfTest()
    assert conf2.cf == 0
