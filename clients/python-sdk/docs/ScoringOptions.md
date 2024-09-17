# ScoringOptions

Scoring options provides ways to modify the sparse or dense vector created for the query in order to change how potential matches are scored. If not specified, this defaults to no modifications.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**fulltext_boost** | [**FullTextBoost**](FullTextBoost.md) |  | [optional] 
**semantic_boost** | [**SemanticBoost**](SemanticBoost.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.scoring_options import ScoringOptions

# TODO update the JSON string below
json = "{}"
# create an instance of ScoringOptions from a JSON string
scoring_options_instance = ScoringOptions.from_json(json)
# print the JSON string representation of the object
print(ScoringOptions.to_json())

# convert the object into a dict
scoring_options_dict = scoring_options_instance.to_dict()
# create an instance of ScoringOptions from a dict
scoring_options_form_dict = scoring_options.from_dict(scoring_options_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


