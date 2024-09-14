# coding: utf-8

"""
    Trieve API

    Trieve OpenAPI Specification. This document describes all of the operations available through the Trieve API.

    The version of the OpenAPI document: 0.11.8
    Contact: developers@trieve.ai
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


import unittest

from trieve_py_client.models.add_chunk_to_group_req_payload import AddChunkToGroupReqPayload

class TestAddChunkToGroupReqPayload(unittest.TestCase):
    """AddChunkToGroupReqPayload unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> AddChunkToGroupReqPayload:
        """Test AddChunkToGroupReqPayload
            include_option is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `AddChunkToGroupReqPayload`
        """
        model = AddChunkToGroupReqPayload()
        if include_optional:
            return AddChunkToGroupReqPayload(
                chunk_id = '',
                chunk_tracking_id = ''
            )
        else:
            return AddChunkToGroupReqPayload(
        )
        """

    def testAddChunkToGroupReqPayload(self):
        """Test AddChunkToGroupReqPayload"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()