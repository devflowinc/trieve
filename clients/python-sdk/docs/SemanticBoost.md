# SemanticBoost

Distance phrase is useful for moving the embedding vector of the chunk in the direction of the distance phrase. I.e. you can push a chunk with a chunk_html of \"iphone\" 25% closer to the term \"flagship\" by using the distance phrase \"flagship\" and a distance factor of 0.25. Conceptually it's drawing a line (euclidean/L2 distance) between the vector for the innerText of the chunk_html and distance_phrase then moving the vector of the chunk_html distance_factor*L2Distance closer to or away from the distance_phrase point along the line between the two points.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**distance_factor** | **float** | Arbitrary float (positive or negative) specifying the multiplicate factor to apply before summing the phrase vector with the chunk_html embedding vector | 
**phrase** | **str** | Terms to embed in order to create the vector which is weighted summed with the chunk_html embedding vector | 

## Example

```python
from trieve_py_client.models.semantic_boost import SemanticBoost

# TODO update the JSON string below
json = "{}"
# create an instance of SemanticBoost from a JSON string
semantic_boost_instance = SemanticBoost.from_json(json)
# print the JSON string representation of the object
print(SemanticBoost.to_json())

# convert the object into a dict
semantic_boost_dict = semantic_boost_instance.to_dict()
# create an instance of SemanticBoost from a dict
semantic_boost_form_dict = semantic_boost.from_dict(semantic_boost_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


