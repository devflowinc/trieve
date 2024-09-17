# SearchesWithoutClicks


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.searches_without_clicks import SearchesWithoutClicks

# TODO update the JSON string below
json = "{}"
# create an instance of SearchesWithoutClicks from a JSON string
searches_without_clicks_instance = SearchesWithoutClicks.from_json(json)
# print the JSON string representation of the object
print(SearchesWithoutClicks.to_json())

# convert the object into a dict
searches_without_clicks_dict = searches_without_clicks_instance.to_dict()
# create an instance of SearchesWithoutClicks from a dict
searches_without_clicks_form_dict = searches_without_clicks.from_dict(searches_without_clicks_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


