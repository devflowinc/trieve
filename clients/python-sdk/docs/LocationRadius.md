# LocationRadius


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**center** | [**GeoInfo**](GeoInfo.md) |  | 
**radius** | **float** |  | 

## Example

```python
from trieve_py_client.models.location_radius import LocationRadius

# TODO update the JSON string below
json = "{}"
# create an instance of LocationRadius from a JSON string
location_radius_instance = LocationRadius.from_json(json)
# print the JSON string representation of the object
print(LocationRadius.to_json())

# convert the object into a dict
location_radius_dict = location_radius_instance.to_dict()
# create an instance of LocationRadius from a dict
location_radius_form_dict = location_radius.from_dict(location_radius_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


