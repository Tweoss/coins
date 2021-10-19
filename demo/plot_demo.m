pkg load statistics;
flips = 50;
resolution = 1/flips;
ticks_x = [0: 1: flips-1]';
ticks_y = [ 0:resolution:1];
a = [0: flips-1];
b = flips-1 - a;
[xx, yy] = meshgrid (ticks_x, ticks_y);
tz = zeros (size (xx));
for i = [1:flips]
	tz(:,i) = betapdf(yy(:, i), a(i)+1, b(i)+1);
end
scatter3 (xx(:), yy(:), tz(:),1, [ -tz(:)], "s");

pause 

for i = [1:flips]
	tz(:,i) = binopdf(i, 20, yy(:, i));
end
scatter3 (xx(:), yy(:), tz(:),1, [-tz(:)], "s");
pause