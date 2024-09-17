# GeoInfo

Location that you want to use as the center of the search.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**lat** | [**GeoTypes**](GeoTypes.md) |  | 
**lon** | [**GeoTypes**](GeoTypes.md) |  | 

## Example

```python
from trieve_py_client.models.geo_info import GeoInfo

# TODO update the JSON string below
json = "{}"
# create an instance of GeoInfo from a JSON string
geo_info_instance = GeoInfo.from_json(json)
# print the JSON string representation of the object
print(GeoInfo.to_json())

# convert the object into a dict
geo_info_dict = geo_info_instance.to_dict()
# create an instance of GeoInfo from a dict
geo_info_form_dict = geo_info.from_dict(geo_info_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


