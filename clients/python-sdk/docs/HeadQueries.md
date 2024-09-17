# HeadQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **int** |  | 
**query** | **str** |  | 

## Example

```python
from trieve_py_client.models.head_queries import HeadQueries

# TODO update the JSON string below
json = "{}"
# create an instance of HeadQueries from a JSON string
head_queries_instance = HeadQueries.from_json(json)
# print the JSON string representation of the object
print(HeadQueries.to_json())

# convert the object into a dict
head_queries_dict = head_queries_instance.to_dict()
# create an instance of HeadQueries from a dict
head_queries_form_dict = head_queries.from_dict(head_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


