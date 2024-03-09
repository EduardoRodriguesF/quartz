tag=$1

git tag --delete $tag
git reset --hard HEAD~
git push --delete origin $tag
gh release delete
gh release delete $tag
git push --force
