# LocationBoundingBox


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bottom_right** | [**GeoInfo**](GeoInfo.md) |  | 
**top_left** | [**GeoInfo**](GeoInfo.md) |  | 

## Example

```python
from trieve_py_client.models.location_bounding_box import LocationBoundingBox

# TODO update the JSON string below
json = "{}"
# create an instance of LocationBoundingBox from a JSON string
location_bounding_box_instance = LocationBoundingBox.from_json(json)
# print the JSON string representation of the object
print(LocationBoundingBox.to_json())

# convert the object into a dict
location_bounding_box_dict = location_bounding_box_instance.to_dict()
# create an instance of LocationBoundingBox from a dict
location_bounding_box_form_dict = location_bounding_box.from_dict(location_bounding_box_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


