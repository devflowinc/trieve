# SearchQueryRating


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**note** | **str** |  | [optional] 
**rating** | **int** |  | 

## Example

```python
from trieve_py_client.models.search_query_rating import SearchQueryRating

# TODO update the JSON string below
json = "{}"
# create an instance of SearchQueryRating from a JSON string
search_query_rating_instance = SearchQueryRating.from_json(json)
# print the JSON string representation of the object
print(SearchQueryRating.to_json())

# convert the object into a dict
search_query_rating_dict = search_query_rating_instance.to_dict()
# create an instance of SearchQueryRating from a dict
search_query_rating_form_dict = search_query_rating.from_dict(search_query_rating_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


