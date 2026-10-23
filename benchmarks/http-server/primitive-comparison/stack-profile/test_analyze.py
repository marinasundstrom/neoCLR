import unittest
from analyze import parse, leaves, display, has_application

class SampleAccounting(unittest.TestCase):
    def tree(self, lines):
        return parse('Call graph:\n'+lines+'\nTotal number in stack (recursive counted multiple):\nignored summary')

    def test_self_samples_include_parent_residual_and_small_leaves(self):
        root=self.tree('    10 Thread_1\n    + 10 outer  (in server) + 8\n    +   6 child  (in server) + 4\n    +   1 tiny  (in server) + 0')[0]
        counts,callers=leaves(root)
        self.assertEqual(dict(counts),{'outer':3,'child':6,'tiny':1})
        self.assertEqual(callers[('child','outer')],6)
        self.assertEqual(sum(counts.values()),10)

    def test_recursion_and_repeated_symbols(self):
        root=self.tree('    8 Thread_1\n    + 5 recur  (in server) + 8\n    +   3 recur  (in server) + 4\n    + 3 recur  (in server) + 0')[0]
        counts,callers=leaves(root)
        self.assertEqual(dict(counts),{'recur':8})
        self.assertEqual(callers[('recur','recur')],3)

    def test_threads_remain_separate(self):
        roots=self.tree('    4 Thread_1 main\n    + 4 wait  (in kernel)\n    3 Thread_2\n    + 3 run  (in server)')
        self.assertEqual(len(roots),2)
        self.assertEqual(dict(leaves(roots[0])[0]),{'wait':4})
        self.assertEqual(dict(leaves(roots[1])[0]),{'run':3})

    def test_overcount_rejected(self):
        root=self.tree('    2 Thread_1\n    + 3 child  (in server)')[0]
        with self.assertRaises(ValueError):leaves(root)

    def test_unknown_format_and_orphan_rejected(self):
        for lines in ['    ??? truncated','    2 run  (in server)','']:
            with self.assertRaises(ValueError):self.tree(lines)

    def test_application_uses_verified_image_name(self):
        root=self.tree('    2 Thread_1\n    + 2 application  (in candidate-w4) + 8')[0]
        self.assertTrue(has_application(root,'candidate-w4'))
        self.assertFalse(has_application(root,'server'))

    def test_bridge_names(self):
        self.assertEqual(display('NeoMetadata_AB.T_53797374656D_54657374.M_52756E'),'System.Test.Run')
        self.assertEqual(display('NeoMetadata_AB.T__54657374..ctor'),'Test..ctor')
        self.assertEqual(display('System.ReadOnlySlice.M_6765745F4974656D'),'System.ReadOnlySlice.get_Item')
        self.assertEqual(display('native_name'),'native_name')

if __name__=='__main__':unittest.main()
