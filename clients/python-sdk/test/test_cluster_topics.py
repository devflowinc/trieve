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

from trieve_py_client.models.cluster_topics import ClusterTopics

class TestClusterTopics(unittest.TestCase):
    """ClusterTopics unit test stubs"""

    def setUp(self):
        pass

    def tearDown(self):
        pass

    def make_instance(self, include_optional) -> ClusterTopics:
        """Test ClusterTopics
            include_option is a boolean, when False only required
            params are included, when True both required and
            optional params are included """
        # uncomment below to create an instance of `ClusterTopics`
        """
        model = ClusterTopics()
        if include_optional:
            return ClusterTopics(
                filter = trieve_py_client.models.cluster_analytics_filter.ClusterAnalyticsFilter(
                    date_range = null, ),
                type = 'cluster_topics'
            )
        else:
            return ClusterTopics(
                type = 'cluster_topics',
        )
        """

    def testClusterTopics(self):
        """Test ClusterTopics"""
        # inst_req_only = self.make_instance(include_optional=False)
        # inst_req_and_optional = self.make_instance(include_optional=True)

if __name__ == '__main__':
    unittest.main()