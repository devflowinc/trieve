# SearchesWithClicks


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.searches_with_clicks import SearchesWithClicks

# TODO update the JSON string below
json = "{}"
# create an instance of SearchesWithClicks from a JSON string
searches_with_clicks_instance = SearchesWithClicks.from_json(json)
# print the JSON string representation of the object
print(SearchesWithClicks.to_json())

# convert the object into a dict
searches_with_clicks_dict = searches_with_clicks_instance.to_dict()
# create an instance of SearchesWithClicks from a dict
searches_with_clicks_form_dict = searches_with_clicks.from_dict(searches_with_clicks_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


