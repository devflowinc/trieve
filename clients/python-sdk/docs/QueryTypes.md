# QueryTypes

Query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.  You can either provide one query, or multiple with weights. Multi-query only works with Semantic Search and is not compatible with cross encoder re-ranking or highlights.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**image_url** | **str** |  | 
**llm_prompt** | **str** |  | [optional] 
**audio_base64** | **str** |  | 

## Example

```python
from trieve_py_client.models.query_types import QueryTypes

# TODO update the JSON string below
json = "{}"
# create an instance of QueryTypes from a JSON string
query_types_instance = QueryTypes.from_json(json)
# print the JSON string representation of the object
print(QueryTypes.to_json())

# convert the object into a dict
query_types_dict = query_types_instance.to_dict()
# create an instance of QueryTypes from a dict
query_types_form_dict = query_types.from_dict(query_types_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


