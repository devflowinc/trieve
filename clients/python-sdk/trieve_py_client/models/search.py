# coding: utf-8

"""
    Trieve API

    Trieve OpenAPI Specification. This document describes all of the operations available through the Trieve API.

    The version of the OpenAPI document: 0.13.0
    Contact: developers@trieve.ai
    Generated by OpenAPI Generator (https://openapi-generator.tech)

    Do not edit the class manually.
"""  # noqa: E501


from __future__ import annotations
import pprint
import re  # noqa: F401
import json

from pydantic import BaseModel, ConfigDict, Field, StrictFloat, StrictInt, StrictStr, field_validator
from typing import Any, ClassVar, Dict, List, Optional, Union
from trieve_py_client.models.clickhouse_search_types import ClickhouseSearchTypes
from trieve_py_client.models.search_query_rating import SearchQueryRating
from typing import Optional, Set
from typing_extensions import Self

class Search(BaseModel):
    """
    Search
    """ # noqa: E501
    event_type: StrictStr
    latency: Optional[Union[StrictFloat, StrictInt]] = Field(default=None, description="Latency of the search")
    query: StrictStr = Field(description="The search query")
    query_rating: Optional[SearchQueryRating] = None
    request_params: Optional[Any] = Field(default=None, description="The request params of the search")
    results: Optional[List[Any]] = Field(default=None, description="The results of the search")
    search_type: Optional[ClickhouseSearchTypes] = None
    top_score: Optional[Union[StrictFloat, StrictInt]] = Field(default=None, description="The top score of the search")
    user_id: Optional[StrictStr] = Field(default=None, description="The user id of the user who made the search")
    __properties: ClassVar[List[str]] = ["event_type", "latency", "query", "query_rating", "request_params", "results", "search_type", "top_score", "user_id"]

    @field_validator('event_type')
    def event_type_validate_enum(cls, value):
        """Validates the enum"""
        if value not in set(['search']):
            raise ValueError("must be one of enum values ('search')")
        return value

    model_config = ConfigDict(
        populate_by_name=True,
        validate_assignment=True,
        protected_namespaces=(),
    )


    def to_str(self) -> str:
        """Returns the string representation of the model using alias"""
        return pprint.pformat(self.model_dump(by_alias=True))

    def to_json(self) -> str:
        """Returns the JSON representation of the model using alias"""
        # TODO: pydantic v2: use .model_dump_json(by_alias=True, exclude_unset=True) instead
        return json.dumps(self.to_dict())

    @classmethod
    def from_json(cls, json_str: str) -> Optional[Self]:
        """Create an instance of Search from a JSON string"""
        return cls.from_dict(json.loads(json_str))

    def to_dict(self) -> Dict[str, Any]:
        """Return the dictionary representation of the model using alias.

        This has the following differences from calling pydantic's
        `self.model_dump(by_alias=True)`:

        * `None` is only added to the output dict for nullable fields that
          were set at model initialization. Other fields with value `None`
          are ignored.
        """
        excluded_fields: Set[str] = set([
        ])

        _dict = self.model_dump(
            by_alias=True,
            exclude=excluded_fields,
            exclude_none=True,
        )
        # override the default output from pydantic by calling `to_dict()` of query_rating
        if self.query_rating:
            _dict['query_rating'] = self.query_rating.to_dict()
        # set to None if latency (nullable) is None
        # and model_fields_set contains the field
        if self.latency is None and "latency" in self.model_fields_set:
            _dict['latency'] = None

        # set to None if query_rating (nullable) is None
        # and model_fields_set contains the field
        if self.query_rating is None and "query_rating" in self.model_fields_set:
            _dict['query_rating'] = None

        # set to None if request_params (nullable) is None
        # and model_fields_set contains the field
        if self.request_params is None and "request_params" in self.model_fields_set:
            _dict['request_params'] = None

        # set to None if results (nullable) is None
        # and model_fields_set contains the field
        if self.results is None and "results" in self.model_fields_set:
            _dict['results'] = None

        # set to None if search_type (nullable) is None
        # and model_fields_set contains the field
        if self.search_type is None and "search_type" in self.model_fields_set:
            _dict['search_type'] = None

        # set to None if top_score (nullable) is None
        # and model_fields_set contains the field
        if self.top_score is None and "top_score" in self.model_fields_set:
            _dict['top_score'] = None

        # set to None if user_id (nullable) is None
        # and model_fields_set contains the field
        if self.user_id is None and "user_id" in self.model_fields_set:
            _dict['user_id'] = None

        return _dict

    @classmethod
    def from_dict(cls, obj: Optional[Dict[str, Any]]) -> Optional[Self]:
        """Create an instance of Search from a dict"""
        if obj is None:
            return None

        if not isinstance(obj, dict):
            return cls.model_validate(obj)

        _obj = cls.model_validate({
            "event_type": obj.get("event_type"),
            "latency": obj.get("latency"),
            "query": obj.get("query"),
            "query_rating": SearchQueryRating.from_dict(obj["query_rating"]) if obj.get("query_rating") is not None else None,
            "request_params": obj.get("request_params"),
            "results": obj.get("results"),
            "search_type": obj.get("search_type"),
            "top_score": obj.get("top_score"),
            "user_id": obj.get("user_id")
        })
        return _obj


