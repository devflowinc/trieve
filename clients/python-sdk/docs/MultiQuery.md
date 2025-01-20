# MultiQuery

MultiQuery allows you to construct a dense vector from multiple queries with a weighted sum. This is useful for when you want to emphasize certain features of the query. This only works with Semantic Search and is not compatible with cross encoder re-ranking or highlights.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**query** | [**SearchModalities**](SearchModalities.md) |  | 
**weight** | **float** | Float value which is applies as a multiplier to the query vector when summing. | 

## Example

```python
from trieve_py_client.models.multi_query import MultiQuery

# TODO update the JSON string below
json = "{}"
# create an instance of MultiQuery from a JSON string
multi_query_instance = MultiQuery.from_json(json)
# print the JSON string representation of the object
print(MultiQuery.to_json())

# convert the object into a dict
multi_query_dict = multi_query_instance.to_dict()
# create an instance of MultiQuery from a dict
multi_query_form_dict = multi_query.from_dict(multi_query_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


